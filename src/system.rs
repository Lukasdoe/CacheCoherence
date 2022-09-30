use crate::bus::Bus;
use crate::core::Core;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;
use std::vec::Vec;

pub struct System {
    cores: Vec<Core>,
    bus: Bus,
    clk: u32,
}

impl System {
    fn new(
        mut record_streams: Vec<RecordStream>,
        protocol: ProtocolKind,
        capacity: usize,
    ) -> System {
        System {
            cores: (1..record_streams.len())
                .map(|_| Core::new(&protocol, capacity, record_streams.remove(0)))
                .collect(),
            bus: Bus::new(),
            clk: 0,
        }
    }
}
