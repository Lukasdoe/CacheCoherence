use crate::bus::{Bus, BusAction};
use clap::ArgEnum;
use serde::Deserialize;
use std::{sync::Arc, vec::Vec};

pub mod dragon;
pub mod mesi;

pub enum ProtocolAction {
    Read,
    Write,
}

pub trait Protocol {
    fn read(&self, addr: usize, hit: bool) -> Vec<BusAction>;
    fn write(&self, addr: usize, hit: bool) -> Vec<BusAction>;
    /// Returns true if the transition is successful, otherwise false and the core has to wait
    fn transition(&mut self, addr: usize, hit: bool, action: ProtocolAction, bus: &mut Bus)
        -> bool;
}
#[derive(Clone, Debug, ArgEnum, Deserialize)]
pub enum ProtocolKind {
    Mesi,
    Dragon,
}

pub struct ProtocolBuilder;

impl ProtocolBuilder {
    pub fn new(
        kind: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
    ) -> Arc<dyn Protocol + Send + Sync> {
        match kind {
            ProtocolKind::Dragon => {
                Arc::new(dragon::Dragon::new(cache_size, associativity, block_size))
            }
            ProtocolKind::Mesi => Arc::new(mesi::Mesi::new(cache_size, associativity, block_size)),
        }
    }
}
