use crate::protocol::{Protocol, ProtocolBuilder, ProtocolKind};
use crate::utils::Counter;
use crate::LOGGER;

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
    // size of the cache in bytes
    cache_size: usize,

    offset_length: usize,
    index_length: usize,
    tag_length: usize,

    num_sets: usize,
    associativity: usize,

    cnt: Counter,
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
        println!("{:?} {:?} {:?}", offset_length, index_length, tag_length);

        #[cfg(debug_assertions)]
        println!("Init cache of size {:?} bytes with {:?} sets of {:?} blocks, each a size of {:?} bytes.",
            cache_size, num_sets, associativity, block_size);

        Cache {
            core_id,

            set_size,
            block_size,
            cache_size,

            cache: vec![vec![PLACEHOLDER_TAG; associativity]; num_sets],
            lru_storage: vec![vec![0; associativity]; num_sets],

            protocol: ProtocolBuilder::new(kind, cache_size, associativity, block_size),

            offset_length,
            index_length,
            tag_length,

            num_sets,
            associativity,

            cnt: Counter::new(),
        }
    }

    fn flat_to_nested(&self, block_idx: usize) -> (usize, usize) {
        let number_of_blocks_in_set = self.set_size / self.block_size;
        let in_set_idx = block_idx % number_of_blocks_in_set;
        let set_idx = (block_idx - in_set_idx) / number_of_blocks_in_set;
        (set_idx, in_set_idx)
    }

    fn nested_to_flat(&self, set_idx: usize, block_idx: usize) -> usize {
        set_idx * self.set_size + block_idx
    }

    #[cfg(debug_assertions)]
    fn print_cache(&self) {
        for set_idx in 0..self.num_sets {
            print!("Set {:?}: ", set_idx);
            for block_idx in 0..self.associativity {
                print!(
                    "(B: {:?}, T: {:?}, L: {:?}); ",
                    block_idx, self.cache[set_idx][block_idx], self.lru_storage[set_idx][block_idx]
                );
            }
            print!("\n");
        }
    }

    /// Advance internal counters.
    /// Returns true iff the cache stalls.
    pub fn update(&mut self) -> bool {
        // TODO: check back with bus
        // For now: no valid dragon supported, no other cache can supply data
        self.update_lru();
        let res = self.cnt.update();

        LOGGER
            .lock()
            .unwrap()
            .log_cache_state(self.core_id, self.cnt.value);
        return res;
    }

    /// Simulate a memory load operation.
    pub fn load(&mut self, addr: u32) {
        assert!(self.cnt.value == 0);

        #[cfg(debug_assertions)]
        println!("Load of addr {:#x} requested (cache).", addr);

        // println!("== Cache State ==");
        // self.print_cache();
        // println!("== =========== ==");

        // TODO: implement protocol

        // cache lookup always takes 1 cycle
        self.cnt.value = 1;

        match self.search(addr) {
            Some((set_idx, block_idx)) => {
                #[cfg(debug_assertions)]
                println!("Hit!");

                self.log_access(set_idx, block_idx);

                LOGGER.lock().unwrap().log_cache_access(
                    self.core_id,
                    true,
                    self.tag(addr),
                    self.index(addr),
                );
                LOGGER
                    .lock()
                    .unwrap()
                    .log_cache_state(self.core_id, self.cnt.value);
            }
            None => {
                #[cfg(debug_assertions)]
                println!("Miss!");

                self.insert_and_evict(addr);
                self.cnt.value += 100;

                LOGGER.lock().unwrap().log_cache_access(
                    self.core_id,
                    false,
                    self.tag(addr),
                    self.index(addr),
                );
                LOGGER
                    .lock()
                    .unwrap()
                    .log_cache_state(self.core_id, self.cnt.value);
            }
        }
    }

    /// Simualate a memory store operation.
    pub fn store(&mut self, addr: u32) {
        #[cfg(debug_assertions)]
        println!("Store to addr {:#x} requested (cache).", addr);

        // write-alloc cache => every write first induces a cache load!
        self.load(addr);

        // TODO: implement protocol
    }

    fn index(&self, addr: u32) -> usize {
        if self.index_length == 0 {
            return 0;
        }
        let right_offset = self.offset_length;
        let mask = (ADDR_MASK_BLANK >> self.tag_length + self.offset_length) << right_offset;
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

    fn search_cache_set(&self, addr: u32, cache_set: &Vec<u32>) -> Option<usize> {
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
            "Found tag {:#x} for addr {:#x} in set {:?}, block id {:?}.",
            self.tag(addr),
            addr,
            set_idx,
            block_idx
        );
        return Some((set_idx, block_idx));
    }

    fn log_access(&mut self, set_idx: usize, block_idx: usize) {
        #[cfg(debug_assertions)]
        println!(
            "Tag {:#x} accessed, resetting LRU val from {:#x} to 0.",
            self.cache[set_idx][block_idx], self.lru_storage[set_idx][block_idx]
        );

        self.lru_storage[set_idx][block_idx] = 0;
    }

    fn update_lru(&mut self) {
        for set_idx in 0..self.num_sets {
            for block_idx in 0..self.associativity {
                self.lru_storage[set_idx][block_idx] = self.lru_storage[set_idx][block_idx] + 1;
            }
        }
    }

    fn insert_and_evict(&mut self, addr_to_load: u32) {
        let new_tag = self.tag(addr_to_load);
        let set_idx = self.index(addr_to_load);

        let cache_set = &mut self.cache[set_idx];
        let lru_cache_set = &mut self.lru_storage[set_idx];
        let (evict_idx, _) = lru_cache_set
            .iter()
            .enumerate()
            .max_by(|(_, i1), (_, i2)| i1.cmp(i2))
            .unwrap();

        let old_tag = cache_set[evict_idx];
        #[cfg(debug_assertions)]
        println!(
            "Tag {:#x} evicted from cache, tag {:#x} loaded. (Set {:?}, Block {:?})",
            old_tag, new_tag, set_idx, evict_idx
        );
        LOGGER.lock().unwrap().log_cache_update(
            self.core_id,
            if old_tag == PLACEHOLDER_TAG {
                None
            } else {
                Some(old_tag)
            },
            new_tag,
            set_idx,
            evict_idx,
        );

        lru_cache_set[evict_idx] = 0;
        cache_set[evict_idx] = new_tag;
    }
}
