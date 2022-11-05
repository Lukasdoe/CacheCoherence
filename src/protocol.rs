use crate::{
    bus::{Bus, BusAction, Task},
    utils::AddressLayout,
};
use clap::ArgEnum;

pub mod dragon;
pub mod mesi;

#[derive(PartialEq, Eq, Debug)]
pub enum ProcessorAction {
    Read,
    Write,
}

pub trait Protocol {
    /// cache_idx contains the flat index of the already stored tag (if stored)
    /// store_idx contains the address at which the tag would be stored (useful if not stored yet)
    fn read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;

    /// cache_idx contains the flat index of the already stored tag (if stored)
    /// store_idx contains the address at which the tag would be stored (useful if not stored yet)
    fn write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        store_idx: usize,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;

    /// Reads bus state and eventually asks to change the current bus state (state transition)
    fn snoop(&mut self, bus: &mut Bus) -> Option<Task>;

    /// applies internal protocol state changes based on final bus state
    fn after_snoop(&mut self, bus: &mut Bus);

    /// true if this cache is the owner of the line. Should cause a flush bus transaction.
    fn writeback_required(&self, cache_idx: usize, tag: u32) -> bool;

    /// mark cache line as invalid
    fn invalidate(&mut self, cache_idx: usize, tag: u32);

    /// true if cache line with the supplied tag and index is shared
    fn is_shared(&self, cache_idx: usize, addr: u32) -> bool;

    /// (debugging only) receive the stored tag for the given cache idx (if any)
    #[cfg(sanity_check)]
    fn sanity_check(&self, cache_idx: usize) -> Option<u32>;

    /// Read broadcast optimization
    fn read_broadcast(&mut self, bus: &mut Bus);
}

#[derive(Clone, Copy, Debug, ArgEnum, PartialEq, Eq)]
pub enum ProtocolKind {
    Mesi,
    Dragon,
}

pub struct ProtocolBuilder;

impl ProtocolBuilder {
    pub fn create(
        core_id: usize,
        kind: &ProtocolKind,
        cache_size: usize,
        block_size: usize,
        associativity: usize,
        addr_layout: &AddressLayout,
    ) -> Box<dyn Protocol> {
        match kind {
            ProtocolKind::Dragon => Box::new(dragon::Dragon::new(
                core_id,
                cache_size,
                block_size,
                associativity,
                addr_layout,
            )),
            ProtocolKind::Mesi => Box::new(mesi::Mesi::new(
                core_id,
                cache_size,
                block_size,
                associativity,
                addr_layout,
            )),
        }
    }
}
