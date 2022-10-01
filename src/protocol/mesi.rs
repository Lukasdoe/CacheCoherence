use super::{Protocol, ProtocolAction};
use crate::bus::{Bus, BusAction};
use std::vec::Vec;

pub enum MesiState {
    M,
    E,
    S,
    I,
}

pub struct Mesi {
    cache_state: Vec<MesiState>,
}

impl Mesi {
    pub fn new(cache_size: usize, associativity: usize, block_size: usize) -> Self {
        Mesi {
            cache_state: Vec::with_capacity(cache_size),
        }
    }
}

impl Protocol for Mesi {
    fn read(&self, addr: usize, hit: bool) -> Vec<BusAction> {
        todo!()
    }

    fn write(&self, addr: usize, hit: bool) -> Vec<BusAction> {
        todo!()
    }

    fn transition(
        &mut self,
        addr: usize,
        hit: bool,
        action: ProtocolAction,
        bus: &mut Bus,
    ) -> bool {
        false
    }
}
