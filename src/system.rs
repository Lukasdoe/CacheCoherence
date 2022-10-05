use crate::bus::Bus;
use crate::core::Core;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;
use crate::LOGGER;

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
        LOGGER.lock().unwrap().log_env(
            format!("{:?}", protocol),
            cache_size,
            associativity,
            block_size,
            record_streams.len(),
        );

        let cores: Vec<Core> = record_streams
            .into_iter()
            .enumerate()
            .map(|(id, stream)| {
                Core::new(&protocol, cache_size, associativity, block_size, stream, id)
            })
            .collect();
        System {
            cores: cores,
            bus: Bus::new(),
            clk: 0,
        }
    }

    /// Execute one simulator step (one cycle) of the multi core system.
    /// Returns true on end of simulation (all instructions executed).
    pub fn update(&mut self) -> bool {
        self.clk += 1;
        LOGGER.lock().unwrap().log_step(self.clk);

        !self
            .cores
            .iter_mut()
            .map(|c| c.step(&mut self.bus))
            .reduce(|acc, core_res| acc || core_res)
            .unwrap_or(false)
    }
}
