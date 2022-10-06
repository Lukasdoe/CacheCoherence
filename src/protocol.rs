use crate::bus::{Bus, BusAction};
use clap::ArgEnum;
use std::vec::Vec;

pub mod dragon;
pub mod mesi;

#[derive(PartialEq)]
pub enum ProcessorAction {
    Read,
    Write,
}

pub trait Protocol {
    fn processor_read(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;
    fn processor_write(
        &mut self,
        tag: u32,
        cache_idx: Option<usize>,
        hit: bool,
        bus: &mut Bus,
    ) -> Option<BusAction>;
    fn bus_snoop(&mut self, bus: &mut Bus);
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
            ProtocolKind::Dragon => Box::new(dragon::Dragon::new(
                core_id,
                cache_size,
                associativity,
                block_size,
            )),
            ProtocolKind::Mesi => Box::new(mesi::Mesi::new(
                core_id,
                cache_size,
                associativity,
                block_size,
            )),
        }
    }
}
