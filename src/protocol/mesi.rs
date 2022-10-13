use super::{ProcessorAction, Protocol};
use crate::bus::{Bus, BusAction, Task};
use std::vec::Vec;

const PLACEHOLDER_TAG: u32 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MesiState {
    M,
    E,
    S,
    I,
}

pub struct Mesi {
    core_id: u32,
    cache_state: Vec<(MesiState, u32)>,
}

impl Mesi {
    pub fn new(core_id: u32, cache_size: usize, block_size: usize) -> Self {
        Mesi {
            core_id,
            cache_state: vec![(MesiState::I, PLACEHOLDER_TAG); cache_size / block_size],
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
        // assert write-allocate cache
        assert!(action != ProcessorAction::Write || hit);
        let (current_state, current_tag) = &self.cache_state[flat_cache_idx.unwrap_or_default()];
        assert!(!hit || *current_tag == tag);
        let (next_state, bus_transaction) = match (current_state, &action, hit) {
            (MesiState::M, ProcessorAction::Read | ProcessorAction::Write, true) => {
                (MesiState::M, None)
            }
            (MesiState::E, ProcessorAction::Write, true) => (MesiState::M, None),
            (MesiState::E, ProcessorAction::Read, true) => (MesiState::E, None),
            (MesiState::S, ProcessorAction::Read, true) => (MesiState::S, None),
            (MesiState::S, ProcessorAction::Write, true) => {
                (MesiState::M, Some(BusAction::BusRdXMem(tag)))
            }

            (MesiState::I, ProcessorAction::Read, true) => {
                (MesiState::E, Some(BusAction::BusRdMem(tag)))
            }
            (MesiState::I, ProcessorAction::Write, true) => {
                (MesiState::M, Some(BusAction::BusRdXMem(tag)))
            }
            (_, ProcessorAction::Read, false) => {
                (MesiState::E, Some(BusAction::BusRdMem(tag)))
            }
            _ => panic!(
                "({:?}) Unresolved processor event: {:?}",
                self.core_id,
                (current_state, action, hit)
            ),
        };
        #[cfg(debug_assertions)]
        println!(
            "({:?}) MESI: Require state transition: {:?} -> {:?}, bus: {:?}",
            self.core_id, current_state, next_state, bus_transaction
        );

        if bus_transaction.is_none() || !bus.occupied() {
            // Cache will issue bus action => already modifiy state
            self.cache_state[flat_store_idx] = (next_state, tag);

            #[cfg(debug_assertions)]
            println!(
                "({:?}) MESI: protocol state successfully updated",
                self.core_id
            );
        } else {
            // else: bus is busy, cache will execute read / write again next cycle. "busy waiting"
            #[cfg(debug_assertions)]
            println!(
                "({:?}) MESI: protocol could not update: bus is busy and required.",
                self.core_id
            );
        }
        bus_transaction
    }

    fn bus_snoop_transition(&mut self, bus: &mut Bus) -> Option<Task> {
        // no active tasks means no snooping
        let mut task = match bus.active_task() {
            Some(t) => t.clone(),
            None => return None,
        };
        if task.issuer_id == self.core_id {
            return None;
        }
        let tag = *task.action;
        // abort if task tag is not cached => we don't care
        let (state, stored_tag) = match self.idx_of_tag(tag) {
            Some(idx) => &mut self.cache_state[idx],
            None => return None,
        };
        assert!(*stored_tag == tag);

        // save for logging purposes:
        let old_state = *state;
        let old_task_action = task.action.clone();
        let old_task_time = task.remaining_cycles;

        match (&task.action, &state) {
            // Event: Someone else wants to read (not exclusive) our modified line
            // => transition from M -> S, flush line to main memory (flush cost + transfer to other
            // core cost)
            (BusAction::BusRdMem(_), MesiState::M) => {
                *state = MesiState::S;
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&BusAction::Flush(0));
            }

            // Event: Someone else wants to readX our modified line
            // => M -> I and Flush
            (BusAction::BusRdXMem(_), MesiState::M) => {
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&BusAction::Flush(0));
            }

            // Event: Someone else wants to read (not X) our exclusive line
            // => E -> S
            (BusAction::BusRdMem(_), MesiState::E) => {
                *state = MesiState::S;
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to readX our exclusive line
            // => E -> I
            (BusAction::BusRdXMem(_), MesiState::E) => {
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to read (not X) our shared line
            // => S -> S and supply line
            (BusAction::BusRdMem(_), MesiState::S) => {
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to readX our shared line
            // => S -> I but supply line
            (BusAction::BusRdXMem(_), MesiState::S) => {
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(tag);
                task.remaining_cycles = Bus::price(&task.action);
            }
            // Ignore bus events that don't change anything
            _ => return None,
        }
        if *state != old_state {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) MESI: Snooping update: State of tag {:x}: {:?} -> {:?}",
                self.core_id, tag, old_state, state
            );
        }
        if task.action != old_task_action || task.remaining_cycles != old_task_time {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) MESI: Snooping update: Task changed: Action {:?} ({:?}) -> Action {:?} ({:?}).", 
                self.core_id, 
                old_task_action, 
                old_task_time, 
                task.action, 
                   task.remaining_cycles
            );
        }
        Some(task)
    }

    fn bus_after_snoop_transition(&mut self, bus: &mut Bus) {
        // no active tasks means no after-snoop
        let task = match bus.active_task() {
            Some(t) => t.clone(),
            None => return,
        };
        // after-snoop only regards actions of other cores on our task
        if task.issuer_id == self.core_id {
            return;
        }
        let tag = *task.action;
        // abort if task tag is not cached => we don't care
        let (state, stored_tag) = match self.idx_of_tag(tag) {
            Some(idx) => &mut self.cache_state[idx],
            None => return,
        };
        assert!(*stored_tag == tag);

        // Event: We read using BusRdMem and some other core changed action to BusRdShared
        // => Value is shared.
        if let (BusAction::BusRdShared(_), MesiState::E) = (&task.action, &state) {
            *state = MesiState::S;
            #[cfg(debug_assertions)]
            println!(
                "({:?}) MESI: After-Snoop update: State of tag {:x}: E -> S",
                self.core_id, tag
            );
        }
    }

    fn idx_of_tag(&self, tag: u32) -> Option<usize> {
        self.cache_state
            .iter()
            .enumerate()
            .find(|(_, (_, stored))| *stored == tag)
            .map(|(idx, _)| idx)
    }
}

impl Protocol for Mesi {
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
}
