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
                // TODO: This writes back values to memory => WRONG!!! should only share!
                (DragonState::M, Some(BusAction::BusUpdMem(tag)))
            }
            (Some((DragonState::Sm, _)), ProcessorAction::Read, true) => (DragonState::Sm, None),
            // check busupd for sharers, if some => DragonState::Sm
            (Some((DragonState::Sm, _)), ProcessorAction::Write, true) => {
                // TODO: This writes back values to memory => WRONG!!! should only share!
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

        // first, decide if we share any data
        match (&task.action, state.as_ref()) {
            // Event: Someone else reads a cache block that we have cached
            // => Change read to shared read and update remaining time.
            // (state transitions are handled later.)
            (BusAction::BusRdMem(_), Some(_)) => {
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else updates a cache block that we have cached
            // => Change update to shared update and update remaining time.
            // (state transitions are handled later.)
            (BusAction::BusUpdMem(_), Some(_)) => {
                task.action = BusAction::BusUpdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }
            _ => (),
        }

        // perform state transitions
        *state = match (&task.action, state.as_ref().map(|value| value.0)) {
            // Event: Bus Read && Line is E
            // => E -> Sc
            (BusAction::BusRdMem(_) | BusAction::BusRdShared(_), Some(DragonState::E)) => {
                Some((DragonState::Sc, tag))
            }

            // Event: Bus Read && Line is Sm
            // => pass
            (BusAction::BusRdMem(_) | BusAction::BusRdShared(_), Some(DragonState::Sm)) => {
                Some((DragonState::Sm, tag))
            }

            // Event: Bus Read && Line is Sc
            // => pass
            (BusAction::BusRdMem(_) | BusAction::BusRdShared(_), Some(DragonState::Sc)) => {
                Some((DragonState::Sc, tag))
            }

            // Event: Bus Read && Line is M
            // => pass
            (BusAction::BusRdMem(_) | BusAction::BusRdShared(_), Some(DragonState::M)) => {
                Some((DragonState::Sm, tag))
            }

            // Event: Bus Update && Line is Sm
            // => Sm -> Sc
            (BusAction::BusUpdMem(_) | BusAction::BusUpdShared(_), Some(DragonState::Sm)) => {
                Some((DragonState::Sc, tag))
            }

            // Event: Bus Update && Line is Sc
            // => pass
            (BusAction::BusUpdMem(_) | BusAction::BusUpdShared(_), Some(DragonState::Sc)) => {
                Some((DragonState::Sc, tag))
            }

            // catch some buggy cases
            (BusAction::BusUpdMem(_) | BusAction::BusUpdShared(_), Some(DragonState::E)) => {
                panic!(
                    "({:?}) Tag {:x}, Task {:?}, State {:?}",
                    self.core_id, tag, task, state
                )
            }
            // Ignore bus events that don't change anything
            (_, d_state) => d_state.map(|s| (s, tag)),
        };
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

            (BusAction::BusRdShared(_), DragonState::E) => {
                *state = Some((DragonState::Sc, tag));
            }

            (BusAction::BusUpdShared(_), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
            }

            (BusAction::BusUpdMem(_), DragonState::M) => {
                bus.clear();
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
