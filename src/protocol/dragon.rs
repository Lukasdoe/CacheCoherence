use super::{ProcessorAction, Protocol};
use crate::bus::{Bus, BusAction, Task};
use crate::system::WORD_SIZE;
use crate::utils::AddressLayout;
use core::panic;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DragonState {
    E,
    Sc,
    Sm,
    M,
}

pub struct Dragon {
    core_id: usize,
    cache_state: Vec<Option<(DragonState, u32)>>,
    block_size: usize,
    associativity: usize,
    addr_layout: AddressLayout,
}

impl Dragon {
    pub fn new(
        core_id: usize,
        cache_size: usize,
        block_size: usize,
        associativity: usize,
        addr_layout: &AddressLayout,
    ) -> Self {
        Dragon {
            core_id,
            cache_state: vec![None; cache_size / block_size],
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
        debug_assert!((hit && flat_cache_idx.is_some()) || (!hit && flat_cache_idx.is_none()));

        // fills current_state with placeholder if hit == false.
        let current_state = &self.cache_state[flat_cache_idx.unwrap_or_default()];

        let (next_state, bus_transaction) = match (current_state, &action, hit) {
            // HIT
            (Some((DragonState::E, _)), ProcessorAction::Read, true) => (DragonState::E, None),
            (Some((DragonState::E, _)), ProcessorAction::Write, true) => (DragonState::M, None),
            (Some((DragonState::Sc, _)), ProcessorAction::Read, true) => (DragonState::Sc, None),
            // check busupd for sharers, if some => DragonState::Sm
            (Some((DragonState::Sc, _)), ProcessorAction::Write, true) => {
                // This bus transaction is cleared at the end of the cycle if no other cache shares
                // the line
                (DragonState::M, Some(BusAction::BusUpdMem(addr, WORD_SIZE)))
            }
            (Some((DragonState::Sm, _)), ProcessorAction::Read, true) => (DragonState::Sm, None),
            // check busupd for sharers, if some => DragonState::Sm
            (Some((DragonState::Sm, _)), ProcessorAction::Write, true) => {
                // This bus transaction is cleared at the end of the cycle if no other cache shares
                // the line
                (DragonState::M, Some(BusAction::BusUpdMem(addr, WORD_SIZE)))
            }
            (Some((DragonState::M, _)), _, true) => (DragonState::M, None),

            // MISS
            // check busupd for sharers, if none => DragonState::E
            (_, ProcessorAction::Read, false) => (
                DragonState::E,
                Some(BusAction::BusRdMem(addr, self.block_size)),
            ),
            (_, ProcessorAction::Write, false) => (
                // dirty hack to signal after_snoop that this is a cold write
                DragonState::E,
                Some(BusAction::BusUpdMem(addr, self.block_size)),
            ),
            _ => panic!(
                "({:?}) Unresolved processor event: {:?}",
                self.core_id,
                (current_state, action, hit)
            ),
        };
        #[cfg(verbose)]
        println!(
            "({:?}) Dragon: Require state transition: {:?} -> {:?}, bus: {:?}",
            self.core_id,
            current_state.map_or(None, |state| Some(state.0)),
            next_state,
            bus_transaction
        );

        if bus_transaction.is_none() || !bus.occupied() {
            // Cache will issue bus action => already modifiy state
            self.cache_state[flat_cache_idx.unwrap_or(flat_store_idx)] =
                Some((next_state, self.addr_layout.tag(addr)));

            #[cfg(verbose)]
            println!(
                "({:?}) Dragon: protocol state successfully updated",
                self.core_id
            );
        } else {
            // else: bus is busy, cache will execute read / write again next cycle. "busy waiting"
            #[cfg(verbose)]
            println!(
                "({:?}) Dragon: protocol could not update: bus is busy and required.",
                self.core_id
            );
        }
        bus_transaction
    }

    fn bus_snoop_transition(&mut self, bus: &mut Bus) -> Option<Task> {
        let mut task = match bus.active_task() {
            Some(t) => t,
            None => return None,
        };
        if task.issuer_id == self.core_id {
            return None;
        }

        let addr = BusAction::extract_addr(task.action);
        let tag = self.addr_layout.tag(addr);
        let idx = match self.idx_of_addr(addr) {
            Some(idx) => idx,
            None => return None,
        };
        let state = self.cache_state[idx].as_mut().unwrap();

        // first, decide if we share any data
        match (&task.action, &state) {
            // Event: Someone else reads a cache block that we have cached
            // => Change read to shared read and update remaining time.
            // (state transitions are handled later.)
            (BusAction::BusRdMem(b_addr, c), _) => {
                debug_assert!(*b_addr == addr);
                task.action = BusAction::BusRdShared(*b_addr, *c);
                task.remaining_cycles = Bus::price(&task.action);
            }

            // Event: Someone else updates a cache block that we have cached
            // => Change update to shared update and update remaining time.
            // (state transitions are handled later.)
            (BusAction::BusUpdMem(b_addr, c), _) => {
                debug_assert!(*b_addr == addr);
                task.action = BusAction::BusUpdShared(*b_addr, *c);
                task.remaining_cycles = Bus::price(&task.action);
            }
            _ => (),
        }

        // perform state transitions
        match (&task.action, state.0) {
            // Event: Bus Read && Line is E
            // => E -> Sc
            (BusAction::BusRdMem(_, _) | BusAction::BusRdShared(_, _), DragonState::E) => {
                *state = (DragonState::Sc, tag)
            }

            // Event: Bus Read && Line is Sm
            // => pass

            // Event: Bus Read && Line is Sc
            // => pass

            // Event: Bus Read && Line is M
            (BusAction::BusRdMem(_, _) | BusAction::BusRdShared(_, _), DragonState::M) => {
                *state = (DragonState::Sm, tag)
            }

            // Event: Bus Update && Line is Sm
            // => Sm -> Sc
            (BusAction::BusUpdMem(_, _) | BusAction::BusUpdShared(_, _), DragonState::Sm) => {
                *state = (DragonState::Sc, tag)
            }

            // Event: Bus Update && Line is Sc
            // => pass

            // Event: Someone else does a cold write (BusRd + BusUpd)
            // and we are in E or M => Sc
            (
                BusAction::BusUpdMem(_, _) | BusAction::BusUpdShared(_, _),
                DragonState::E | DragonState::M,
            ) => *state = (DragonState::Sc, tag),

            // Ignore bus events that don't change anything
            _ => (),
        };
        Some(*task)
    }

    fn bus_after_snoop_transition(&mut self, bus: &mut Bus) {
        let task = match bus.active_task() {
            Some(t) => t,
            None => return,
        };
        if task.issuer_id != self.core_id {
            return;
        }

        let addr = BusAction::extract_addr(task.action);
        let tag = self.addr_layout.tag(addr);
        let idx = match self.idx_of_addr(addr) {
            Some(idx) => idx,
            None => return,
        };
        let state = &mut self.cache_state[idx];

        match (
            task.action,
            state.as_ref().map_or(DragonState::E, |value| value.0),
        ) {
            // Event: We read using BusRdMem and some other core changed action to BusRdShared
            // => Value is shared.
            (BusAction::BusRdShared(_, _), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
            }

            (BusAction::BusRdShared(_, _), DragonState::E) => {
                *state = Some((DragonState::Sc, tag));
            }

            (BusAction::BusUpdShared(_, _), DragonState::M) => {
                *state = Some((DragonState::Sm, tag));
            }

            (BusAction::BusUpdMem(_, _), DragonState::M) => {
                bus.clear();
            }

            // cold writes are signaled by not setting the state to the appropriate M
            (BusAction::BusUpdMem(_, _), DragonState::E) => {
                // BusUpd not required, BusRd time should be counted though.
                task.action = BusAction::BusRdMem(addr, self.block_size);
                task.remaining_cycles = Bus::price(&task.action);
                *state = Some((DragonState::M, tag));
            }
            (BusAction::BusUpdShared(_, _), DragonState::E) => {
                // BusUpd and BusRd required => adjust time.
                task.action = BusAction::BusUpdShared(addr, self.block_size);
                task.remaining_cycles = Bus::price(&task.action)
                    + Bus::price(&BusAction::BusRdShared(addr, self.block_size));
                *state = Some((DragonState::Sm, tag));
            }
            _ => (),
        }
    }

    fn idx_of_addr(&self, addr: u32) -> Option<usize> {
        let start_idx = self.addr_layout.index(addr) * self.associativity;
        let tag = self.addr_layout.tag(addr);
        (start_idx..(start_idx + self.associativity))
            .find(|&i| self.cache_state[i].map_or(false, |(_, t)| t == tag))
    }
}

impl Protocol for Dragon {
    fn read(
        &mut self,
        addr: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(addr, cache_idx, store_idx, hit, ProcessorAction::Read, bus)
    }

    fn write(
        &mut self,
        addr: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction> {
        self.processor_transition(addr, cache_idx, store_idx, hit, ProcessorAction::Write, bus)
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

    fn is_shared(&self, mut cache_idx: usize, addr: u32) -> bool {
        if cache_idx == std::usize::MAX {
            cache_idx = self.idx_of_addr(addr).unwrap();
        }
        assert!(self.cache_state[cache_idx].is_some());
        let (state, stored_tag) = self.cache_state[cache_idx].unwrap();
        assert!(stored_tag == self.addr_layout.tag(addr));
        matches!(state, DragonState::Sc | DragonState::Sm)
    }

    #[cfg(sanity_check)]
    fn sanity_check(&self, cache_idx: usize) -> Option<u32> {
        self.cache_state[cache_idx].map(|(_, tag)| tag)
    }

    fn invalidate(&mut self, cache_idx: usize, tag: u32) {
        debug_assert!(self.cache_state[cache_idx].unwrap().1 == tag);
        self.cache_state[cache_idx] = None
    }

    fn read_broadcast(&mut self, _: &mut Bus) {
        panic!("Read broadcast optimization cannot be used with dragon protocol.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CACHE_SIZE: usize = 16;
    const BLOCK_SIZE: usize = 4;
    const ASSOCIATIVITY: usize = 1;

    fn addr_layout(cache_size: usize, block_size: usize, associativity: usize) -> AddressLayout {
        let set_size = associativity * block_size;
        let num_sets = cache_size / set_size;

        // as integer logs are currently unstable, we have to be ugly
        let offset_length = ((block_size / 4) as f64).log2() as usize;
        let index_length = (num_sets as f64).log2() as usize;
        let tag_length = 32 - (offset_length + index_length);

        AddressLayout::new(
            offset_length,
            index_length,
            tag_length,
            set_size,
            block_size,
        )
    }

    #[test]
    fn invalid_to_exclusive_to_exclusive_to_modified() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(0, action);
        }

        let task = protocol.snoop(&mut bus);
        assert!(task.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        protocol.after_snoop(&mut bus);
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let action = protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let action = protocol.write(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );
    }

    #[test]
    fn invalid_to_modified_to_modified() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.write(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let action = protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let action = protocol.write(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );
    }

    #[test]
    fn invalid_to_shared_clean_to_shared_clean() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let mut other_protocol = Dragon::new(1, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(other_protocol.cache_state[store_idx], None);
        let action = other_protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        let action = protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        let action = other_protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );
    }

    #[test]
    fn exclusive_to_shared_modified_to_shared_modified_and_switch() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let mut other_protocol = Dragon::new(1, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(other_protocol.cache_state[store_idx], None);
        let action = other_protocol.write(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );

        while bus.occupied() {
            bus.update();
        }

        let action = protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        let action = other_protocol.read(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_none());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );

        let action = protocol.write(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(0, action);
        }

        let task = other_protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        protocol.after_snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );
    }

    #[test]
    fn modified_to_shared_modified() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.write(addr, None, store_idx, false, &mut bus);

        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        assert!(action.is_some());
        if let Some(action) = action {
            assert_eq!(action, BusAction::BusUpdMem(addr, BLOCK_SIZE));
            bus.put_on(0, action);
        }

        protocol.after_snoop(&mut bus);
        bus.clear();

        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );

        let mut other_protocol = Dragon::new(1, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(other_protocol.cache_state[store_idx], None);
        let action = other_protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        while bus.occupied() {
            bus.update();
        }

        let mut another_protocol = Dragon::new(2, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(another_protocol.cache_state[store_idx], None);
        let action = another_protocol.write(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            another_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(2, action);
        }

        let _ = protocol.snoop(&mut bus);
        let task = other_protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        another_protocol.after_snoop(&mut bus);
        assert_eq!(
            another_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );
    }

    #[test]
    fn shared_modified_to_modified() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let mut other_protocol = Dragon::new(1, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(other_protocol.cache_state[store_idx], None);
        let action = other_protocol.write(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );

        while bus.occupied() {
            bus.update();
        }

        // invalidate
        protocol.cache_state[store_idx] = None;

        let action = other_protocol.write(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_none());
        assert_eq!(protocol.cache_state[store_idx], None);

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );
    }

    #[test]
    fn shared_clean_to_modified() {
        let layout = addr_layout(CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY);
        let mut protocol = Dragon::new(0, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);
        let mut bus = Bus::new();

        let addr = 0x100;
        let set_idx = layout.index(addr);
        let block_idx = 0;
        let store_idx = layout.nested_to_flat(set_idx, block_idx);

        assert_eq!(protocol.cache_state[store_idx], None);
        let action = protocol.read(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        let mut other_protocol = Dragon::new(1, CACHE_SIZE, BLOCK_SIZE, ASSOCIATIVITY, &layout);

        assert_eq!(other_protocol.cache_state[store_idx], None);
        let action = other_protocol.write(addr, None, store_idx, false, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::E, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(1, action);
        }

        let task = protocol.snoop(&mut bus);

        assert!(task.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sc, layout.tag(addr))
        );

        other_protocol.after_snoop(&mut bus);

        assert_eq!(
            other_protocol.cache_state[store_idx].unwrap(),
            (DragonState::Sm, layout.tag(addr))
        );

        while bus.occupied() {
            bus.update();
        }

        // invalidate
        other_protocol.cache_state[store_idx] = None;

        let action = protocol.write(addr, Some(store_idx), store_idx, true, &mut bus);
        assert!(action.is_some());
        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );

        if let Some(action) = action {
            bus.put_on(0, action);
        }

        let task = other_protocol.snoop(&mut bus);

        assert!(task.is_none());
        assert_eq!(other_protocol.cache_state[store_idx], None);

        protocol.after_snoop(&mut bus);

        assert_eq!(
            protocol.cache_state[store_idx].unwrap(),
            (DragonState::M, layout.tag(addr))
        );
    }
}
