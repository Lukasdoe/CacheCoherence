use crate::bus::{Bus, BusAction, Task};
use clap::ArgEnum;
pub mod dragon;
pub mod mesi;

#[derive(PartialEq)]
pub enum ProcessorAction {
    Read,
    Write,
}

pub trait Protocol {
    fn read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;
    fn write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;
    fn snoop(&mut self, bus: &mut Bus) -> Option<Task>;
    fn after_snoop(&mut self, bus: &mut Bus);
}

#[derive(Clone, Debug, ArgEnum)]
pub enum ProtocolKind {
    Mesi,
    Dragon,
}

pub struct ProtocolBuilder;

impl ProtocolBuilder {
    pub fn new(
        core_id: u32,
        kind: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
    ) -> Box<dyn Protocol> {
        match kind {
            ProtocolKind::Dragon => Box::new(dragon::Dragon::new(core_id, cache_size, block_size)),
            ProtocolKind::Mesi => Box::new(mesi::Mesi::new(
                core_id,
                cache_size,
                associativity,
                block_size,
            )),
        }
    }
}
