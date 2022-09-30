use crate::protocol::{Protocol, ProtocolBuilder, ProtocolKind};
const ADDR_LEN: u32 = 32;
const ADDR_MASK_BLANK: u32 = (2_u64.pow(ADDR_LEN) - 1) as u32;

pub struct Cache {
    cache: Vec<Vec<Vec<u32>>>,
    protocol: Box<dyn Protocol>,
    set_size: usize,
    block_size: usize,
    cache_size: usize,

    offset_length: usize,
    index_length: usize,
    tag_length: usize,
}

impl Cache {
    pub fn new(
        capacity: usize,
        associativity: usize,
        block_size: usize,
        kind: &ProtocolKind,
    ) -> Self {
        assert!(capacity % 2 == 0);
        assert!(associativity % 2 == 0);
        assert!(block_size % 2 == 0);

        let cache_set_size = associativity * block_size;

        // as integer logs are currently unstable, we have to be ugly
        let offset_length = (block_size as f64).log2() as usize;
        let index_length = ((capacity / cache_set_size) as f64).log2() as usize;
        Cache {
            set_size: cache_set_size,
            block_size: block_size,
            cache_size: capacity,
            cache: vec![vec![vec![0; block_size]; associativity]; capacity / cache_set_size],
            protocol: ProtocolBuilder::new(kind, capacity, associativity, block_size),

            offset_length: offset_length,
            index_length: index_length,
            tag_length: offset_length + index_length,
        }
    }

    fn offset(&self, addr: u32) -> usize {
        ((addr as u32) & (ADDR_MASK_BLANK >> (self.tag_length + self.index_length))) as usize
    }

    fn index(&self, addr: u32) -> usize {
        ((addr as u32)
            & ((ADDR_MASK_BLANK >> (self.tag_length + self.offset_length)) << self.offset_length))
            as usize
    }

    fn tag(&self, addr: u32) -> u32 {
        (addr as u32)
            & ((ADDR_MASK_BLANK >> (self.offset_length + self.index_length))
                << (self.offset_length + self.index_length))
    }

    fn search_cache_set(&self, addr: u32, cache_set: &Vec<Vec<u32>>) -> Option<(usize, usize)> {
        for (pos, block) in cache_set.iter().enumerate() {
            match block.binary_search(&self.tag(addr)) {
                Ok(block_pos) => return Some((pos, block_pos)),
                Err(_) => (),
            }
        }
        None
    }

    pub fn search(&self, addr: u32) -> Option<usize> {
        let set_idx = self.index(addr);
        let cache_set = match self.cache.get(set_idx) {
            Some(s) => s,
            None => return None,
        };
        let (block_idx, tag_idx) = match self.search_cache_set(addr, cache_set) {
            Some(s) => s,
            None => return None,
        };
        return Some(set_idx * self.set_size + block_idx * self.block_size + tag_idx * 1);
    }
}
