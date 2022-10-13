use crate::bus::{Bus, BusAction, Task};
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
}

#[derive(Clone, Debug, ArgEnum)]
pub enum ProtocolKind {
    Mesi,
    Dragon,
}

pub struct ProtocolBuilder;

impl ProtocolBuilder {
    pub fn create(
        core_id: u32,
        kind: &ProtocolKind,
        cache_size: usize,
        block_size: usize,
    ) -> Box<dyn Protocol> {
        match kind {
            ProtocolKind::Dragon => Box::new(dragon::Dragon::new(core_id, cache_size, block_size)),
            ProtocolKind::Mesi => Box::new(mesi::Mesi::new(core_id, cache_size, block_size)),
        }
    }
}
