use crate::bus::{Bus, BusAction};

use super::{Protocol, ProtocolAction};

pub enum DragonState {
    E,
    Sc,
    Sm,
    M,
}

pub struct Dragon {
    cache_state: Vec<DragonState>,
}

impl Dragon {
    pub fn new(cache_size: usize, associativity: usize, block_size: usize) -> Self {
        Dragon {
            cache_state: Vec::with_capacity(cache_size),
        }
    }
}

impl Protocol for Dragon {
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
        let current_state = &self.cache_state[addr];

        let (next_state, bus_transaction) = match (current_state, action, hit) {
            (DragonState::E, ProtocolAction::Read, true) => (DragonState::E, None),
            (DragonState::E, ProtocolAction::Read, false) => (DragonState::E, Some(0)),
            (DragonState::E, ProtocolAction::Write, true) => (DragonState::M, None),
            (DragonState::E, ProtocolAction::Write, false) => (DragonState::E, None),
            (DragonState::Sc, ProtocolAction::Read, true) => (DragonState::E, None),
            (DragonState::Sc, ProtocolAction::Read, false) => (DragonState::E, None),
            (DragonState::Sc, ProtocolAction::Write, true) => (DragonState::E, None),
            (DragonState::Sc, ProtocolAction::Write, false) => (DragonState::E, None),
            (DragonState::Sm, ProtocolAction::Read, true) => (DragonState::E, None),
            (DragonState::Sm, ProtocolAction::Read, false) => (DragonState::E, None),
            (DragonState::Sm, ProtocolAction::Write, true) => (DragonState::E, None),
            (DragonState::Sm, ProtocolAction::Write, false) => (DragonState::E, None),
            (DragonState::M, ProtocolAction::Read, true) => (DragonState::E, None),
            (DragonState::M, ProtocolAction::Read, false) => (DragonState::E, None),
            (DragonState::M, ProtocolAction::Write, true) => (DragonState::E, None),
            (DragonState::M, ProtocolAction::Write, false) => (DragonState::E, None),
        };

        if bus_transaction.is_none() {
            // We don't need the bus for the state update
            self.cache_state[addr] = next_state;
            true
        } else if !bus.occupied() {
            // We need the bus and have to check that the bus is free
            true
        } else {
            false
        }
    }
}
