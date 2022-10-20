use crate::analyzer::Analyzable;
use crate::bus::{BusAction, Task};
use crate::protocol::{ProcessorAction, Protocol, ProtocolBuilder, ProtocolKind};
use crate::utils::AddressLayout;
use crate::Bus;
use std::collections::VecDeque;

const PLACEHOLDER_TAG: u32 = 0;

pub struct Cache {
    core_id: usize,
    cache: Vec<Vec<u32>>,
    lru: Lru,
    protocol: Box<dyn Protocol>,
    addr_layout: AddressLayout,

    // size of a block in bytes
    block_size: usize,

    // Queue of waiting instructions (address, action)
    scheduled_instructions: VecDeque<(u32, ProcessorAction)>,
    stats: CacheStats,
}

#[derive(Default)]
struct CacheStats {
    pub num_data_cache_misses: usize,
    pub num_data_cache_hits: usize,
    pub num_private_data_access: usize,
    pub num_shared_data_access: usize,
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

        #[cfg(verbose)]
        println!("({:?}) Init cache of size {:?} bytes with {:?} sets of {:?} blocks, each a size of {:?} bytes.",
            core_id, cache_size, num_sets, associativity, block_size);

        let addr_layout = AddressLayout::new(
            offset_length,
            index_length,
            tag_length,
            set_size,
            block_size,
        );
        Cache {
            core_id,
            block_size,

            cache: vec![vec![PLACEHOLDER_TAG; associativity]; num_sets],
            lru: Lru::new(num_sets, associativity),
            protocol: ProtocolBuilder::create(
                core_id,
                kind,
                cache_size,
                block_size,
                associativity,
                &addr_layout,
            ),
            addr_layout,
            scheduled_instructions: VecDeque::new(),
            stats: CacheStats::default(),
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
            if let BusAction::Flush(_, _) = action {
                return;
            }
            let addr = BusAction::extract_addr(action);
            if self.protocol.is_shared(std::usize::MAX, addr) {
                self.stats.num_shared_data_access += 1;
            } else {
                self.stats.num_private_data_access += 1;
            }
        }
    }

    /// Returns true if the operation could be completed / scheduled
    fn internal_load(&mut self, addr: u32, bus: &mut Bus) -> bool {
        #[cfg(verbose)]
        println!(
            "({:?}) Load of addr {:#x} requested (cache).",
            self.core_id, addr
        );

        let store_idx = self.search(addr);
        let (evict_set, evict_block) = self.get_evict_index(addr);
        let evict_tag = self.cache[evict_set][evict_block];
        let flat_store_idx = store_idx
            .map(|(set_idx, block_idx)| self.addr_layout.nested_to_flat(set_idx, block_idx));
        let flat_evict_idx = self.addr_layout.nested_to_flat(evict_set, evict_block);

        if store_idx != None {
            // TODO: is a hit on an invalid cache line also a hit?
            #[cfg(verbose)]
            println!("({:?}) Hit.", self.core_id);
        } else {
            #[cfg(verbose)]
            println!("({:?}) Miss.", self.core_id);

            let writeback_required = store_idx.is_none()
                && evict_tag != PLACEHOLDER_TAG
                && self.protocol.writeback_required(flat_evict_idx, evict_tag);

            // first execute write_back, then care about loading / coherence of new value
            if writeback_required {
                #[cfg(verbose)]
                println!(
                    "({:?}) Writeback required (cache line occupied and cache is owner)",
                    self.core_id
                );

                if bus.occupied() {
                    #[cfg(verbose)]
                    println!("({:?}) Bus is busy, write back postponed", self.core_id);

                    return false;
                }
                bus.put_on(self.core_id, BusAction::Flush(evict_tag, self.block_size));
                self.protocol.invalidate(flat_evict_idx, evict_tag);

                // clear cache for later insert
                self.cache[evict_set][evict_block] = PLACEHOLDER_TAG;
                // set Lru to low value, so this cell will be evicted next.
                self.lru.eliminate(evict_set, evict_block);

                #[cfg(verbose)]
                println!("({:?}) Writeback commissioned.", self.core_id);
                return false;
            }
        }
        // --- after this point, the optional write-back is already done => load new value!
        let bus_action = self.protocol.read(
            addr,
            flat_store_idx,
            flat_evict_idx,
            store_idx.is_some(),
            bus,
        );

        if let Some(action) = bus_action {
            if bus.occupied() {
                #[cfg(verbose)]
                println!(
                    "({:?}) Cache load required the bus ({:?}), which is busy.",
                    self.core_id, action
                );

                return false;
            }
            #[cfg(verbose)]
            println!(
                "({:?}) Cache load executed bus transaction {:?}",
                self.core_id, action
            );
            bus.put_on(self.core_id, action);
        }

        if let Some((set_idx, block_idx)) = store_idx {
            self.log_access(set_idx, block_idx);
            self.stats.num_data_cache_hits += 1;

            if bus_action.is_none() {
                if self.protocol.is_shared(flat_store_idx.unwrap(), addr) {
                    self.stats.num_shared_data_access += 1;
                } else {
                    self.stats.num_private_data_access += 1;
                }
            }
        } else {
            // the memory for this is already flushed to main memory
            // TODO: should we flush to other cores? => nah
            self.insert_and_evict(addr);
            self.stats.num_data_cache_misses += 1;
        }

        #[cfg(verbose)]
        println!(
            "({:?}) Cache load of addr {:#x} successfully completed.",
            self.core_id, addr
        );
        true
    }

    /// Returns true if the operation could be completed / scheduled
    fn internal_store(&mut self, addr: u32, bus: &mut Bus) -> bool {
        #[cfg(verbose)]
        println!(
            "({:?}) Store to addr {:#x} requested (cache).",
            self.core_id, addr
        );

        // write-alloc cache => test if addr is cached
        let cache_idx_opt = self.search(addr);
        if cache_idx_opt.is_none() {
            #[cfg(verbose)]
            println!(
                "({:?}) Store addr {:#x} is not cached, scheduling read for next cycle.",
                self.core_id, addr
            );

            self.scheduled_instructions
                .push_front((addr, ProcessorAction::Read));
            return false;
        }
        #[cfg(verbose)]
        println!(
            "({:?}) Store addr {:#x} is cached, continuing store.",
            self.core_id, addr
        );
        let cache_idx = cache_idx_opt.unwrap();
        let flat_cache_idx = self.addr_layout.nested_to_flat(cache_idx.0, cache_idx.1);
        let bus_action = self
            .protocol
            .write(addr, Some(flat_cache_idx), flat_cache_idx, true, bus);

        if let Some(action) = bus_action {
            if bus.occupied() {
                #[cfg(verbose)]
                println!(
                    "({:?}) Cache store required the bus ({:?}), which is busy.",
                    self.core_id, action
                );
                return false;
            }
            #[cfg(verbose)]
            println!(
                "({:?}) Cache store executed bus transaction {:?}",
                self.core_id, action
            );
            bus.put_on(self.core_id, action);
        }
        #[cfg(verbose)]
        println!(
            "({:?}) Cache store of addr {:#x} successfully completed.",
            self.core_id, addr
        );
        true
    }

    fn search_cache_set(&self, addr: u32, cache_set: &[u32]) -> Option<usize> {
        let tag = self.addr_layout.tag(addr);
        cache_set.iter().position(|block_tag| *block_tag == tag)
    }

    /// Test if supplied addr's tag is currently cached.
    /// Returns first index of block in flat cache.
    fn search(&self, addr: u32) -> Option<(usize, usize)> {
        let set_idx = self.addr_layout.index(addr);
        let cache_set = self.cache.get(set_idx)?;
        let block_idx = self.search_cache_set(addr, cache_set)?;

        #[cfg(verbose)]
        println!(
            "({:?}) Found tag {:#x} for addr {:#x} in set {:?}, block id {:?}.",
            self.core_id,
            self.addr_layout.tag(addr),
            addr,
            set_idx,
            block_idx
        );
        Some((set_idx, block_idx))
    }

    fn log_access(&mut self, set_idx: usize, block_idx: usize) {
        #[cfg(verbose)]
        println!(
            "({:?}) Tag {:#x} accessed, resetting Lru val from {:#x} to 0.",
            self.core_id, self.cache[set_idx][block_idx], self.lru_storage[set_idx][block_idx]
        );

        self.lru.update(set_idx, block_idx);
    }

    fn get_evict_index(&self, addr_to_load: u32) -> (usize, usize) {
        let set_idx = self.addr_layout.index(addr_to_load);
        self.lru.get_lru_idx(set_idx)
    }

    fn insert_and_evict(&mut self, addr_to_load: u32) {
        let new_tag = self.addr_layout.tag(addr_to_load);
        let set_idx = self.addr_layout.index(addr_to_load);
        let evict_idx = self.get_evict_index(addr_to_load).1;
        let cache_set = &mut self.cache[set_idx];

        #[cfg(verbose)]
        let old_tag = cache_set[evict_idx];
        #[cfg(verbose)]
        println!(
            "({:?}) Tag {:#x} evicted from cache, tag {:#x} loaded. (Set {:?}, Block {:?})",
            self.core_id, old_tag, new_tag, set_idx, evict_idx
        );

        self.lru.update(set_idx, evict_idx);
        cache_set[evict_idx] = new_tag;
    }

    #[cfg(sanity_check)]
    pub fn sanity_check(&self) {
        for (set_idx, set) in self.cache.iter().enumerate() {
            for (block_idx, block) in set.iter().enumerate() {
                let prot_tag = self
                    .protocol
                    .sanity_check(self.addr_layout.nested_to_flat(set_idx, block_idx));
                if prot_tag.is_some() && *block != PLACEHOLDER_TAG {
                    assert_eq!(prot_tag.unwrap(), *block)
                }
            }
        }
    }
}

struct Lru {
    storage: Vec<Vec<usize>>,
    cnt: usize,
}

impl Lru {
    pub fn new(num_sets: usize, associativity: usize) -> Lru {
        Lru {
            storage: vec![vec![0; associativity]; num_sets],
            cnt: 1,
        }
    }

    pub fn update(&mut self, set_idx: usize, block_idx: usize) {
        self.storage[set_idx][block_idx] = self.cnt;
        self.cnt += 1;
    }

    pub fn eliminate(&mut self, set_idx: usize, block_idx: usize) {
        self.storage[set_idx][block_idx] = 0;
    }

    pub fn get_lru_idx(&self, set_idx: usize) -> (usize, usize) {
        (
            set_idx,
            self.storage[set_idx]
                .iter()
                .enumerate()
                .min_by(|(_, i1), (_, i2)| i1.cmp(i2))
                .unwrap()
                .0,
        )
    }
}

impl Analyzable for Cache {
    fn report(&self, stats: &mut crate::analyzer::Stats) {
        let c_stats = &mut stats.cores[self.core_id];
        c_stats.num_data_cache_hits = self.stats.num_data_cache_hits;
        c_stats.num_data_cache_misses = self.stats.num_data_cache_misses;
        stats.num_private_data_access += self.stats.num_private_data_access;
        stats.num_shared_data_access += self.stats.num_shared_data_access;
    }
}
