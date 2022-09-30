use crate::bus::Bus;
use crate::core::Core;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;

pub struct System {
    cores: Vec<Core>,
    bus: Bus,
    clk: u32,
}

impl System {
    pub fn new(
        protocol: &ProtocolKind,
        capacity: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
    ) -> Self {
        let cores: Vec<Core> = record_streams
            .into_iter()
            .map(|stream| Core::new(&protocol, capacity, associativity, block_size, stream))
            .collect();
        System {
            cores: cores,
            bus: Bus::new(),
            clk: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        for core in self.cores.iter_mut() {
            core.step(&mut self.bus);
        }



        false
    }
}
