use core::panic;

use crate::bus::{Bus, BusAction, Task};

use super::{ProcessorAction, Protocol};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    pub fn new(core_id: u32, cache_size: usize, block_size: usize) -> Self {
        Dragon {
            core_id,
            cache_state: vec![None; cache_size / block_size],
        }
    }

    fn processor_transition(
        &mut self,
        tag: u32,
        flat_cache_idx: Option<usize>,
        flat_store_idx: usize,
        hit: bool,
        action: ProcessorAction,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        assert!(action != ProcessorAction::Write || hit);

        // fills current_state with placeholder if hit == false.
        let current_state = &self.cache_state[flat_cache_idx.unwrap_or_default()];

        let (next_state, bus_transaction) = match (current_state, &action, hit) {
            // HIT
            (Some((DragonState::E, _)), ProcessorAction::Read, true) => (DragonState::E, None),
            (Some((DragonState::E, _)), ProcessorAction::Write, true) => (DragonState::M, None),
            (Some((DragonState::Sc, _)), ProcessorAction::Read, true) => (DragonState::Sc, None),
            // check busupd for sharers, if some => DragonState::Sm
            (Some((DragonState::Sc, _)), ProcessorAction::Write, true) => {
                (DragonState::M, Some(BusAction::BusUpdMem(tag)))
            }
            (Some((DragonState::Sm, _)), ProcessorAction::Read, true) => (DragonState::Sm, None),
            // check busupd for sharers, if some => DragonState::Sm
            (Some((DragonState::Sm, _)), ProcessorAction::Write, true) => {
                (DragonState::M, Some(BusAction::BusUpdMem(tag)))
            }
            (Some((DragonState::M, _)), _, true) => (DragonState::M, None),

            // MISS
            // check busupd for sharers, if none => DragonState::E
            (_, ProcessorAction::Read, false) => (DragonState::E, Some(BusAction::BusRdMem(tag))),
            // this can not occur:
            // (_, ProcessorAction::Write, false) => (DragonState::Sm, None),
            _ => panic!(
                "({:?}) Unresolved processor event: {:?}",
                self.core_id,
                (current_state, action, hit)
            ),
        };
        #[cfg(debug_assertions)]
        println!(
            "({:?}) Dragon: Require state transition: {:?} -> {:?}, bus: {:?}",
            self.core_id,
            current_state.map_or(None, |state| Some(state.0)),
            next_state,
            bus_transaction
        );

        if bus_transaction.is_none() || !bus.occupied() {
            // Cache will issue bus action => already modifiy state
            self.cache_state[flat_cache_idx.unwrap_or(flat_store_idx)] = Some((next_state, tag));

            #[cfg(debug_assertions)]
            println!(
                "({:?}) Dragon: protocol state successfully updated",
                self.core_id
            );
        } else {
            // else: bus is busy, cache will execute read / write again next cycle. "busy waiting"
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Dragon: protocol could not update: bus is busy and required.",
                self.core_id
            );
        }
        bus_transaction
    }

    fn bus_snoop_transition(&mut self, bus: &mut Bus) -> Option<Task> {
        let mut task = match bus.active_task() {
            Some(t) => t.clone(),
            None => return None,
        };
        if task.issuer_id == self.core_id {
            return None;
        }
        let tag = *task.action;
        let state = match self.idx_of_tag(tag) {
            Some(idx) => &mut self.cache_state[idx],
            None => return None,
        };
        match (
            &task.action,
            state.as_ref().map_or(DragonState::E, |value| value.0),
        ) {
            // Event: Someone else reads our exclusive cache block
            // => transition from E -> Sc, update bus transaction to shared, update task time
            (BusAction::BusRdMem(_), DragonState::E) => {
                *state = Some((DragonState::Sc, tag));
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else reads a block that we have in Sm-state
            // => Flush (in this case: change to read shared)
            (BusAction::BusRdMem(_), DragonState::Sm) => {
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else updates a block that we have in Sm
            // => transition from Sm -> Sc, update bus transaction to shared, update task time
            (BusAction::BusUpdMem(_), DragonState::Sm) => {
                *state = Some((DragonState::Sc, tag));
                task.action = BusAction::BusUpdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else reads a block that we have in M-state
            // => transition from M -> Sm, change memory read transaction to update and flush
            // We don't actually perform a flush, we just add the flush-time to the current read operation
            (BusAction::BusRdMem(_), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // catch some buggy cases
            (BusAction::BusRdShared(_), DragonState::E) => {
                panic!()
            }

            // Ignore bus events that don't change anything
            _ => return None,
        }
        Some(task)
    }

    fn bus_after_snoop_transition(&mut self, bus: &mut Bus) {
        let task = match bus.active_task() {
            Some(t) => t.clone(),
            None => return,
        };
        if task.issuer_id != self.core_id {
            return;
        }
        let tag = *task.action;
        let state = match self.idx_of_tag(tag) {
            Some(idx) => &mut self.cache_state[idx],
            None => return,
        };
        match (
            &task.action,
            state.as_ref().map_or(DragonState::E, |value| value.0),
        ) {
            // Event: We read using BusRdMem and some other core changed action to BusRdShared
            // => Value is shared.
            (BusAction::BusRdShared(_), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
            }

            (BusAction::BusUpdShared(_), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
            }

            (BusAction::BusRdShared(_), DragonState::E) => {
                *state = Some((DragonState::Sc, tag));
            }
            _ => (),
        }
    }

    fn idx_of_tag(&self, tag: u32) -> Option<usize> {
        self.cache_state
            .iter()
            .enumerate()
            .find(|(_, stored)| stored.is_some() && stored.as_ref().unwrap().1 == tag)
            .map(|(idx, _)| idx)
    }
}

impl Protocol for Dragon {
    fn read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(tag, cache_idx, store_idx, hit, ProcessorAction::Read, bus)
    }

    fn write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(tag, cache_idx, store_idx, hit, ProcessorAction::Write, bus)
    }

    fn snoop(&mut self, bus: &mut Bus) -> Option<Task> {
        self.bus_snoop_transition(bus)
    }

    fn after_snoop(&mut self, bus: &mut Bus) {
        self.bus_after_snoop_transition(bus)
    }

    fn writeback_required(&self, cache_idx: usize, tag: u32) -> bool {
        assert!(self.cache_state[cache_idx].is_some());
        let (state, stored_tag) = self.cache_state[cache_idx].unwrap();
        assert!(stored_tag == tag);
        state == DragonState::M || state == DragonState::Sm
    }
}
