use super::{ProcessorAction, Protocol};
use crate::bus::{Bus, Task};
use crate::system::WORD_SIZE;
use crate::utils::AddressLayout;
use shared::bus::BusAction;
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
    core_id: usize,
    cache_state: Vec<(MesiState, u32)>,
    block_size: usize,
    associativity: usize,
    addr_layout: AddressLayout,
}

impl Mesi {
    pub fn new(
        core_id: usize,
        cache_size: usize,
        block_size: usize,
        associativity: usize,
        addr_layout: &AddressLayout,
    ) -> Self {
        Mesi {
            core_id,
            cache_state: vec![(MesiState::I, PLACEHOLDER_TAG); cache_size / block_size],
            block_size,
            associativity,
            addr_layout: *addr_layout,
        }
    }

    fn processor_transition(
        &mut self,
        addr: u32,
        flat_cache_idx: Option<usize>,
        flat_store_idx: usize,
        hit: bool,
        action: ProcessorAction,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        // assert write-allocate cache
        assert!(action != ProcessorAction::Write || hit);
        let (current_state, current_tag) = &self.cache_state[flat_cache_idx.unwrap_or_default()];
        assert!(!hit || *current_tag == self.addr_layout.tag(addr));
        let (next_state, bus_transaction) = match (current_state, &action, hit) {
            // HIT
            (MesiState::M, ProcessorAction::Read | ProcessorAction::Write, true) => {
                (MesiState::M, None)
            }
            (MesiState::E, ProcessorAction::Write, true) => (MesiState::M, None),
            (MesiState::E, ProcessorAction::Read, true) => (MesiState::E, None),
            (MesiState::S, ProcessorAction::Read, true) => (MesiState::S, None),
            (MesiState::S, ProcessorAction::Write, true) => {
                (MesiState::M, Some(BusAction::BusRdXMem(addr, WORD_SIZE)))
            }

            (MesiState::I, ProcessorAction::Read, true) => (
                MesiState::E,
                Some(BusAction::BusRdMem(addr, self.block_size)),
            ),
            (MesiState::I, ProcessorAction::Write, true) => (
                MesiState::M,
                Some(BusAction::BusRdXMem(addr, self.block_size)),
            ),

            // MISS
            (_, ProcessorAction::Read, false) => (
                MesiState::E,
                Some(BusAction::BusRdMem(addr, self.block_size)),
            ),
            _ => panic!(
                "({:?}) Unresolved processor event: {:?}",
                self.core_id,
                (current_state, action, hit)
            ),
        };
        #[cfg(verbose)]
        println!(
            "({:?}) MESI: Require state transition: {:?} -> {:?}, bus: {:?}",
            self.core_id, current_state, next_state, bus_transaction
        );

        if bus_transaction.is_none() || !bus.occupied() {
            // Cache will issue bus action || no bus action required => already modify state
            self.cache_state[flat_cache_idx.unwrap_or(flat_store_idx)] =
                (next_state, self.addr_layout.tag(addr));
        } else {
            // else: bus is busy, cache will execute read / write again next cycle. "busy waiting"
            #[cfg(verbose)]
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
            Some(t) => t,
            None => return None,
        };
        if task.issuer_id == self.core_id {
            return None;
        }
        let addr = BusAction::extract_addr(task.action);
        let tag = self.addr_layout.tag(addr);
        // abort if task tag is not cached => we don't care
        let (state, stored_tag) = match self.idx_of_addr(addr) {
            Some(idx) => &mut self.cache_state[idx],
            None => return None,
        };
        assert!(*stored_tag == tag);

        // save for logging purposes:
        let old_state = *state;
        let old_task_action = task.action;
        let old_task_time = task.remaining_cycles;

        match (task.action, &state) {
            // Event: Someone else wants to read (not exclusive) our modified line
            // => transition from M -> S, flush line to main memory (flush cost + transfer to other
            // core cost)
            (BusAction::BusRdMem(b_addr, c), MesiState::M) => {
                debug_assert!(b_addr == addr);
                *state = MesiState::S;
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&BusAction::Flush(0, self.block_size));
            }

            // Event: Someone else wants to readX our modified line
            // => M -> I and Flush
            (BusAction::BusRdXMem(b_addr, c), MesiState::M) => {
                debug_assert!(b_addr == addr);
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&BusAction::Flush(0, self.block_size));
            }

            // Event: Someone else wants to read (not X) our exclusive line
            // => E -> S
            (BusAction::BusRdMem(b_addr, c), MesiState::E) => {
                debug_assert!(b_addr == addr);
                *state = MesiState::S;
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to readX our exclusive line
            // => E -> I
            (BusAction::BusRdXMem(b_addr, c), MesiState::E) => {
                debug_assert!(b_addr == addr);
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to read (not X) our shared line
            // => S -> S and supply line
            (BusAction::BusRdMem(b_addr, c), MesiState::S) => {
                debug_assert!(b_addr == addr);
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else wants to readX our shared line
            // => S -> I but supply line
            (BusAction::BusRdXMem(b_addr, c), MesiState::S) => {
                debug_assert!(b_addr == addr);
                *state = MesiState::I;
                task.action = BusAction::BusRdShared(b_addr, c);
                task.remaining_cycles = Bus::price(&task.action);
            }
            // Ignore bus events that don't change anything
            _ => return None,
        }
        if *state != old_state {
            #[cfg(verbose)]
            println!(
                "({:?}) MESI: Snooping update: State of tag {:x}: {:?} -> {:?}",
                self.core_id, tag, old_state, state
            );
        }
        if task.action != old_task_action || task.remaining_cycles != old_task_time {
            #[cfg(verbose)]
            println!(
                "({:?}) MESI: Snooping update: Task changed: Action {:?} ({:?}) -> Action {:?} ({:?}).",
                self.core_id,
                old_task_action,
                old_task_time,
                task.action,
                   task.remaining_cycles
            );
        }
        Some(*task)
    }

    fn bus_after_snoop_transition(&mut self, bus: &mut Bus) {
        // no active tasks means no after-snoop
        let task = match bus.active_task() {
            Some(t) => t,
            None => return,
        };
        // after-snoop only regards actions of other cores on our task
        if task.issuer_id == self.core_id {
            return;
        }
        let addr = BusAction::extract_addr(task.action);
        let tag = self.addr_layout.tag(addr);
        // abort if task tag is not cached => we don't care
        let (state, stored_tag) = match self.idx_of_addr(addr) {
            Some(idx) => &mut self.cache_state[idx],
            None => return,
        };
        assert!(*stored_tag == tag);

        // Event: We read using BusRdMem and some other core changed action to BusRdShared
        // => Value is shared.
        if let (BusAction::BusRdShared(_, _), MesiState::E) = (&task.action, &state) {
            *state = MesiState::S;
            #[cfg(verbose)]
            println!(
                "({:?}) MESI: After-Snoop update: State of tag {:x}: E -> S",
                self.core_id, tag
            );
        }
    }

    fn idx_of_addr(&self, addr: u32) -> Option<usize> {
        let start_idx = self.addr_layout.index(addr);
        let tag = self.addr_layout.tag(addr);
        (start_idx..(start_idx + self.associativity)).find(|&i| self.cache_state[i].1 == tag)
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

    fn writeback_required(&self, cache_idx: usize, tag: u32) -> bool {
        let (state, stored_tag) = self.cache_state[cache_idx];
        assert!(stored_tag == tag);
        state == MesiState::M
    }

    fn is_shared(&self, mut cache_idx: usize, addr: u32) -> bool {
        if cache_idx == std::usize::MAX {
            cache_idx = self.idx_of_addr(addr).unwrap();
        }
        let (state, stored_tag) = self.cache_state[cache_idx];
        assert!(stored_tag == self.addr_layout.tag(addr));
        assert!(state != MesiState::I);
        matches!(state, MesiState::S)
    }

    #[cfg(sanity_check)]
    fn sanity_check(&self, cache_idx: usize) -> Option<u32> {
        Some(self.cache_state[cache_idx].1)
    }

    fn invalidate(&mut self, cache_idx: usize, tag: u32) {
        debug_assert!(self.cache_state[cache_idx].1 == tag);
        self.cache_state[cache_idx] = (MesiState::I, PLACEHOLDER_TAG)
    }
}
