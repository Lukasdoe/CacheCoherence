use crate::bus::{Bus, BusAction, Task};

use super::{ProcessorAction, Protocol};

#[derive(PartialEq, Clone)]
pub enum DragonState {
    E,
    Sc,
    Sm,
    M,
}

pub struct Dragon {
    core_id: u32,
    cache_state: Vec<Option<(DragonState, u32)>>,
}

impl Dragon {
    pub fn new(core_id: u32, cache_size: usize, associativity: usize, block_size: usize) -> Self {
        Dragon {
            core_id,
            cache_state: vec![None; (cache_size / block_size)],
        }
    }

    fn processor_transition(
        &mut self,
        tag: u32,
        flat_cache_idx: Option<usize>,
        hit: bool,
        action: ProcessorAction,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        assert!(action != ProcessorAction::Write || hit);

        // fills current_state with placeholder if hit == false.
        let (current_state, _) =
            &self.cache_state[flat_cache_idx.unwrap_or_default()].unwrap_or((DragonState::E, 0));

        let (next_state, bus_transaction) = match (current_state, action, hit) {
            // HIT
            (DragonState::E, ProcessorAction::Read, true) => (DragonState::E, None),
            (DragonState::E, ProcessorAction::Write, true) => (DragonState::M, None),

            (DragonState::Sc, ProcessorAction::Read, true) => (DragonState::Sc, None),

            // check busupd for sharers, if some => DragonState::Sm
            (DragonState::Sc, ProcessorAction::Write, true) => {
                (DragonState::M, Some(BusAction::BusUpdMem(tag)))
            }

            (DragonState::Sm, ProcessorAction::Read, true) => (DragonState::Sm, None),

            // check busupd for sharers, if some => DragonState::Sm
            (DragonState::Sm, ProcessorAction::Write, true) => {
                (DragonState::M, Some(BusAction::BusUpdMem(tag)))
            }

            (DragonState::M, _, true) => (DragonState::M, None),

            // MISS
            // check busupd for sharers, if none => DragonState::M
            (_, ProcessorAction::Read, false) => (DragonState::Sc, Some(BusAction::BusRdMem(tag))),
            // this can not occur:
            // (_, ProcessorAction::Write, false) => (DragonState::Sm, None),
            _ => panic!("Unresolved processor event."),
        };

        if bus_transaction.is_none() || !bus.occupied() {
            // Cache will issue bus action => already modifiy state
            self.cache_state[flat_cache_idx.unwrap_or_default()] = Some((next_state, tag));
        }
        // else: bus is busy, cache will execute read / write again next cycle. "busy waiting"
        return bus_transaction;
    }

    fn idx_of_tag(&self, tag: u32) -> Option<usize> {
        self.cache_state
            .iter()
            .enumerate()
            .find(|(_, stored)| stored.is_some() && stored.unwrap().1 == tag)
            .map_or(None, |(idx, _)| Some(idx))
    }

    fn bus_transition(&mut self, bus: &mut Bus) {
        let task = bus.active_task();
        if task.is_none() {
            return;
        }

        match task.unwrap() {
            // Event: We read using BusRdMem and some other core changed action to BusRdShared
            // => Value is shared.
            Task {
                issuer_id: id,
                action: BusAction::BusRdShared(tag),
                ..
            } => {
                let idx = self.idx_of_tag(tag);
                if id == self.core_id
                    && idx.is_some()
                    && self.cache_state[idx.unwrap()].0 == DragonState::M
                {
                    self.cache_state[idx.unwrap()] = (DragonState::Sm, tag)
                }
            }

            // Event: Someone else reads our exclusive cache block
            // => transition from E => Sc
            Task {
                issuer_id: id,
                action: BusAction::BusRdShared(tag),
                ..
            } => {
                let idx = self.idx_of_tag(tag);
                if id != self.core_id
                    && idx.is_some()
                    && self.cache_state[idx.unwrap()].0 == DragonState::E
                {
                    self.cache_state[idx.unwrap()] = (DragonState::Sc, tag)
                }
            }

            // Event: Someone else updates a block that we have in Sm
            // => transition from Sm => Sc
            Task {
                issuer_id: id,
                action: BusAction::BusRdShared(tag),
                ..
            } => {
                let idx = self.idx_of_tag(tag);
                if id != self.core_id
                    && idx.is_some()
                    && self.cache_state[idx.unwrap()].0 == DragonState::Sm
                {
                    self.cache_state[idx.unwrap()] = (DragonState::Sc, tag)
                }
            }

            // Event: Someone else reads a block that we in M-state
            // => transition from M => Sm and Flush
            Task {
                issuer_id: id,
                action: BusAction::BusRdShared(tag),
                ..
            } => {
                let idx = self.idx_of_tag(tag);
                if id != self.core_id
                    && idx.is_some()
                    && self.cache_state[idx.unwrap()].0 == DragonState::M
                {
                    self.cache_state[idx.unwrap()] = (DragonState::Sm, tag)
                }
                // TODO: Flush
            }

            // Event: Someone else reads a block that we have in Sm-state
            // => Flush
            Task {
                issuer_id: id,
                action: BusAction::BusRdShared(tag),
                ..
            } => {
                let idx = self.idx_of_tag(tag);
                if id != self.core_id
                    && idx.is_some()
                    && self.cache_state[idx.unwrap()].0 == DragonState::M
                {
                    // TODO: Flush
                }
            }

            // Ignore bus events that don't change anything
            _ => (),
        }
    }
}

impl Protocol for Dragon {
    fn processor_read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(tag, cache_idx, hit, ProcessorAction::Read, bus)
    }

    fn processor_write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(tag, cache_idx, hit, ProcessorAction::Write, bus)
    }

    fn bus_snoop(&mut self, bus: &mut Bus) {
        self.bus_transition(bus)
    }
}
