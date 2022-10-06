use super::{ProcessorAction, Protocol};
use crate::bus::{Bus, BusAction};
use std::vec::Vec;

pub enum MesiState {
    M,
    E,
    S,
    I,
}

pub struct Mesi {
    core_id: u32,
    cache_state: Vec<MesiState>,
}

impl Mesi {
    pub fn new(core_id: u32, cache_size: usize, associativity: usize, block_size: usize) -> Self {
        Mesi {
            core_id,
            cache_state: Vec::with_capacity(cache_size),
        }
    }
}

impl Protocol for Mesi {
    fn processor_read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        todo!()
    }

    fn processor_write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        todo!()
    }

    fn bus_snoop(&mut self, bus: &mut Bus) {
        todo!()
    }
}
