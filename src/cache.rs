use crate::bus::Task;
use crate::protocol::{ProcessorAction, Protocol, ProtocolBuilder, ProtocolKind};
use crate::{Bus, LOGGER};
use logger::*;
use shared::bus::BusAction;
use std::collections::VecDeque;

const ADDR_LEN: u32 = 32;
const ADDR_MASK_BLANK: u32 = (2_u64.pow(ADDR_LEN) - 1) as u32;
const PLACEHOLDER_TAG: u32 = 0;

pub struct Cache {
    core_id: usize,
    cache: Vec<Vec<u32>>,
    lru_storage: Vec<Vec<usize>>,

    protocol: Box<dyn Protocol>,

    // size of a cache set in bytes
    set_size: usize,
    // size of a block in bytes
    block_size: usize,

    offset_length: usize,
    index_length: usize,
    tag_length: usize,

    num_sets: usize,
    associativity: usize,

    // Queue of waiting instructions (address, action)
    scheduled_instructions: VecDeque<(u32, ProcessorAction)>,
}

impl Cache {
    pub fn new(
        core_id: usize,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        kind: &ProtocolKind,
    ) -> Self {
        let set_size = associativity * block_size;
        let num_sets = cache_size / set_size;

        // as integer logs are currently unstable, we have to be ugly
        let offset_length = ((block_size / 4) as f64).log2() as usize;
        let index_length = (num_sets as f64).log2() as usize;
        let tag_length = 32 - (offset_length + index_length);

        #[cfg(debug_assertions)]
        println!("({:?}) Init cache of size {:?} bytes with {:?} sets of {:?} blocks, each a size of {:?} bytes.",
            core_id, cache_size, num_sets, associativity, block_size);

        Cache {
            core_id,

            set_size,
            block_size,

            cache: vec![vec![PLACEHOLDER_TAG; associativity]; num_sets],
            lru_storage: vec![vec![0; associativity]; num_sets],

            protocol: ProtocolBuilder::create(core_id, kind, cache_size, block_size),

            offset_length,
            index_length,
            tag_length,

            num_sets,
            associativity,

            scheduled_instructions: VecDeque::new(),
        }
    }

    /// Simulate a memory load operation.
    pub fn load(&mut self, addr: u32) {
        self.scheduled_instructions
            .push_back((addr, ProcessorAction::Read));
    }

    /// Simualate a memory store operation.
    pub fn store(&mut self, addr: u32) {
        self.scheduled_instructions
            .push_back((addr, ProcessorAction::Write));
    }

    /// Advance internal counters.
    /// Returns true iff the cache stalls.
    pub fn update(&mut self, bus: &mut Bus) -> bool {
        self.update_lru();

        // we currently write to the bus => better back off until this is finished
        if bus
            .active_task()
            .map_or(false, |t| t.issuer_id == self.core_id)
        {
            return true;
        }

        if let Some((addr, action)) = self.scheduled_instructions.pop_front() {
            // loads are always stored as next instruction, stores might place loads in front and
            // are therefore always the last instruction.
            match action {
                ProcessorAction::Read => {
                    if !self.internal_load(addr, bus) {
                        self.scheduled_instructions
                            .push_front((addr, ProcessorAction::Read));
                    }
                }
                ProcessorAction::Write => {
                    if !self.internal_store(addr, bus) {
                        self.scheduled_instructions
                            .push_back((addr, ProcessorAction::Write));
                    }
                }
            }
            assert!(self.scheduled_instructions.len() < 3);
            return true;
        }
        false
    }

    pub fn snoop(&mut self, bus: &mut Bus) {
        if let Some(task) = self.protocol.snoop(bus) {
            *bus.active_task().unwrap() = task
        }
    }

    pub fn after_snoop(&mut self, bus: &mut Bus) {
        self.protocol.after_snoop(bus);
        if bus.active_task().map_or(
            false,
            |Task {
                 issuer_id,
                 remaining_cycles,
                 ..
             }| *issuer_id == self.core_id && *remaining_cycles == 0,
        ) {
            let action = bus.active_task().unwrap().action;
            let tag = BusAction::extract_tag(action);
            LOGGER.write(if self.protocol.is_shared(std::usize::MAX, tag) {
                LogEntry::CacheSharedAccess(CacheSharedAccess {
                    id: self.core_id,
                    tag,
                })
            } else {
                LogEntry::CachePrivateAccess(CachePrivateAccess {
                    id: self.core_id,
                    tag,
                })
            });
        }
    }

    fn nested_to_flat(&self, set_idx: usize, block_idx: usize) -> usize {
        set_idx * (self.set_size / self.block_size) + block_idx
    }

    /// Returns true if the operation could be completed / scheduled
    fn internal_load(&mut self, addr: u32, bus: &mut Bus) -> bool {
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Load of addr {:#x} requested (cache).",
            self.core_id, addr
        );

        let loaded_tag = self.tag(addr);
        let store_idx = self.search(addr);
        let (evict_set, evict_block) = self.get_evict_index(addr);
        let evict_tag = self.cache[evict_set][evict_block];
        let flat_store_idx =
            store_idx.map(|(set_idx, block_idx)| self.nested_to_flat(set_idx, block_idx));
        let flat_evict_idx = self.nested_to_flat(evict_set, evict_block);

        if store_idx != None {
            // TODO: is a hit on an invalid cache line also a hit?
            #[cfg(debug_assertions)]
            println!("({:?}) Hit.", self.core_id);
        } else {
            #[cfg(debug_assertions)]
            println!("({:?}) Miss.", self.core_id);

            let writeback_required = store_idx.is_none()
                && evict_tag != PLACEHOLDER_TAG
                && self.protocol.writeback_required(flat_evict_idx, evict_tag);

            // first execute write_back, then care about loading / coherence of new value
            if writeback_required {
                #[cfg(debug_assertions)]
                println!(
                    "({:?}) Writeback required (cache line occupied and cache is owner)",
                    self.core_id
                );

                if bus.occupied() {
                    #[cfg(debug_assertions)]
                    println!("({:?}) Bus is busy, write back postponed", self.core_id);

                    return false;
                }
                bus.put_on(self.core_id, BusAction::Flush(evict_tag, self.block_size));

                // clear cache for later insert
                self.cache[evict_set][evict_block] = PLACEHOLDER_TAG;
                // set LRU to high value, so this cell will be evicted next.
                self.lru_storage[evict_set][evict_block] = usize::MAX / 2;

                #[cfg(debug_assertions)]
                println!("({:?}) Writeback commissioned.", self.core_id);
                return false;
            }
        }
        // --- after this point, the optional write-back is already done => load new value!
        let bus_action = self.protocol.read(
            loaded_tag,
            flat_store_idx,
            flat_evict_idx,
            store_idx.is_some(),
            bus,
        );

        if let Some(action) = bus_action {
            if bus.occupied() {
                #[cfg(debug_assertions)]
                println!(
                    "({:?}) Cache load required the bus ({:?}), which is busy.",
                    self.core_id, action
                );

                return false;
            }
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Cache load executed bus transaction {:?}",
                self.core_id, action
            );
            bus.put_on(self.core_id, action);
        }

        if let Some((set_idx, block_idx)) = store_idx {
            self.log_access(set_idx, block_idx);
            LOGGER.write(LogEntry::CacheHit(CacheHit {
                id: self.core_id,
                tag: loaded_tag,
            }));

            if bus_action.is_none() {
                LOGGER.write(
                    if self.protocol.is_shared(flat_store_idx.unwrap(), loaded_tag) {
                        LogEntry::CacheSharedAccess(CacheSharedAccess {
                            id: self.core_id,
                            tag: loaded_tag,
                        })
                    } else {
                        LogEntry::CachePrivateAccess(CachePrivateAccess {
                            id: self.core_id,
                            tag: loaded_tag,
                        })
                    },
                );
            }
        } else {
            // the memory for this is already flushed to main memory
            // TODO: should we flush to other cores? => nah
            self.insert_and_evict(addr);
            LOGGER.write(LogEntry::CacheMiss(CacheMiss {
                id: self.core_id,
                tag: self.tag(addr),
            }));
        }

        #[cfg(debug_assertions)]
        println!(
            "({:?}) Cache load of addr {:#x} successfully completed.",
            self.core_id, addr
        );
        true
    }

    /// Returns true if the operation could be completed / scheduled
    fn internal_store(&mut self, addr: u32, bus: &mut Bus) -> bool {
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Store to addr {:#x} requested (cache).",
            self.core_id, addr
        );

        // write-alloc cache => test if addr is cached
        let cache_idx_opt = self.search(addr);
        if cache_idx_opt.is_none() {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Store addr {:#x} is not cached, scheduling read for next cycle.",
                self.core_id, addr
            );

            self.scheduled_instructions
                .push_front((addr, ProcessorAction::Read));
            return false;
        }
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Store addr {:#x} is cached, continuing store.",
            self.core_id, addr
        );
        let cache_idx = cache_idx_opt.unwrap();
        let flat_cache_idx = self.nested_to_flat(cache_idx.0, cache_idx.1);
        let bus_action = self.protocol.write(
            self.tag(addr),
            Some(flat_cache_idx),
            flat_cache_idx,
            true,
            bus,
        );

        if let Some(action) = bus_action {
            if bus.occupied() {
                #[cfg(debug_assertions)]
                println!(
                    "({:?}) Cache store required the bus ({:?}), which is busy.",
                    self.core_id, action
                );
                return false;
            }
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Cache store executed bus transaction {:?}",
                self.core_id, action
            );
            bus.put_on(self.core_id, action);
        }
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Cache store of addr {:#x} successfully completed.",
            self.core_id, addr
        );
        true
    }

    fn index(&self, addr: u32) -> usize {
        if self.index_length == 0 {
            return 0;
        }
        let right_offset = self.offset_length;
        let mask = (ADDR_MASK_BLANK >> (self.tag_length + self.offset_length)) << right_offset;
        let masked_addr = (addr as u32) & mask;
        (masked_addr >> right_offset) as usize
    }

    fn tag(&self, addr: u32) -> u32 {
        if self.tag_length == 0 {
            return 0;
        }
        let right_offset = self.offset_length + self.index_length;
        addr >> right_offset
    }

    fn search_cache_set(&self, addr: u32, cache_set: &[u32]) -> Option<usize> {
        let tag = self.tag(addr);
        cache_set.iter().position(|&block_tag| block_tag == tag)
    }

    /// Test if supplied addr's tag is currently cached.
    /// Returns first index of block in flat cache.
    fn search(&self, addr: u32) -> Option<(usize, usize)> {
        let set_idx = self.index(addr);
        let cache_set = self.cache.get(set_idx)?;
        let block_idx = self.search_cache_set(addr, cache_set)?;

        #[cfg(debug_assertions)]
        println!(
            "({:?}) Found tag {:#x} for addr {:#x} in set {:?}, block id {:?}.",
            self.core_id,
            self.tag(addr),
            addr,
            set_idx,
            block_idx
        );
        Some((set_idx, block_idx))
    }

    fn log_access(&mut self, set_idx: usize, block_idx: usize) {
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Tag {:#x} accessed, resetting LRU val from {:#x} to 0.",
            self.core_id, self.cache[set_idx][block_idx], self.lru_storage[set_idx][block_idx]
        );

        self.lru_storage[set_idx][block_idx] = 0;
    }

    fn update_lru(&mut self) {
        for set_idx in 0..self.num_sets {
            for block_idx in 0..self.associativity {
                self.lru_storage[set_idx][block_idx] += 1;
            }
        }
    }

    fn get_evict_index(&self, addr_to_load: u32) -> (usize, usize) {
        let set_idx = self.index(addr_to_load);
        return (
            set_idx,
            self.lru_storage[set_idx]
                .iter()
                .enumerate()
                .max_by(|(_, i1), (_, i2)| i1.cmp(i2))
                .unwrap()
                .0,
        );
    }

    fn insert_and_evict(&mut self, addr_to_load: u32) {
        let new_tag = self.tag(addr_to_load);
        let set_idx = self.index(addr_to_load);
        let evict_idx = self.get_evict_index(addr_to_load).1;

        let cache_set = &mut self.cache[set_idx];
        let lru_cache_set = &mut self.lru_storage[set_idx];

        #[cfg(debug_assertions)]
        let old_tag = cache_set[evict_idx];
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Tag {:#x} evicted from cache, tag {:#x} loaded. (Set {:?}, Block {:?})",
            self.core_id, old_tag, new_tag, set_idx, evict_idx
        );

        lru_cache_set[evict_idx] = 0;
        cache_set[evict_idx] = new_tag;
    }
}
