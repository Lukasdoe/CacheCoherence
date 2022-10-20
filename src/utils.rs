pub struct Counter {
    pub value: u32,
}

impl Counter {
    pub fn new() -> Self {
        Counter { value: 0 }
    }

    /// Update counter. Returns false if counter reached 0.
    pub fn update(&mut self) -> bool {
        match self.value {
            0 => false,
            c => {
                self.value = c - 1;
                true
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AddressLayout {
    offset_length: usize,
    index_length: usize,
    tag_length: usize,

    // size of a cache set in bytes
    set_size: usize,
    // size of a block in bytes
    block_size: usize,
}

const ADDR_LEN: u32 = 32;
const ADDR_MASK_BLANK: u32 = (2_u64.pow(ADDR_LEN) - 1) as u32;

impl AddressLayout {
    pub fn new(
        offset_length: usize,
        index_length: usize,
        tag_length: usize,
        set_size: usize,
        block_size: usize,
    ) -> AddressLayout {
        AddressLayout {
            offset_length,
            index_length,
            tag_length,
            set_size,
            block_size,
        }
    }

    pub fn index(&self, addr: u32) -> usize {
        if self.index_length == 0 {
            return 0;
        }
        let right_offset = self.offset_length;
        let mask = (ADDR_MASK_BLANK >> (self.tag_length + self.offset_length)) << right_offset;
        let masked_addr = (addr as u32) & mask;
        (masked_addr >> right_offset) as usize
    }

    pub fn tag(&self, addr: u32) -> u32 {
        if self.tag_length == 0 {
            return 0;
        }
        let right_offset = self.offset_length + self.index_length;
        addr >> right_offset
    }

    pub fn nested_to_flat(&self, set_idx: usize, block_idx: usize) -> usize {
        set_idx * (self.set_size / self.block_size) + block_idx
    }
}
