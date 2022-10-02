use crate::bus::Bus;
use crate::core::{Core, CoreState};
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;

#[derive(Default)]
pub struct System {
    cores: Vec<Core>,
    bus: Bus,
    clk: u32,
}

impl System {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
    ) -> Self {
        let mut system = System::default();
        system.load(
            protocol,
            cache_size,
            associativity,
            block_size,
            record_streams,
        );
        system
    }

    pub fn load(
        &mut self,
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
    ) {
        self.cores = record_streams
            .into_iter()
            .enumerate()
            .map(|(id, stream)| {
                Core::new(&protocol, cache_size, associativity, block_size, stream, id)
            })
            .collect();
    }

    /// Execute one simulator step (one cycle) of the multi core system.
    /// Returns true on end of simulation (all instructions executed).
    pub fn update(&mut self) -> bool {
        self.clk += 1;
        !self
            .cores
            .iter_mut()
            .map(|c| c.step(&mut self.bus))
            .reduce(|acc, core_res| acc || core_res)
            .unwrap_or(false)
    }

    pub fn info(&self) -> Vec<&CoreState> {
        self.cores.iter().map(|c| c.state()).collect()
    }
}
