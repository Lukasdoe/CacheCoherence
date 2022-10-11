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
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
    ) -> Self {
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
        #[cfg(debug_assertions)]
        println!("Step {:?}", self.clk);

        self.bus.update();

        // "at_least_one_core_is_still_working"
        let mut alocisw = false;

        // run 1: parse new instructions / update state
        for core in self.cores.iter_mut() {
            alocisw = core.step(&mut self.bus) || alocisw;
        }

        // run 2: snoop other cores' actions
        for core in self.cores.iter_mut() {
            core.snoop(&mut self.bus);
        }

        // run 3: cleanup after bus snooping
        for core in self.cores.iter_mut() {
            core.after_snoop(&mut self.bus);
        }

        if !alocisw {
            println!("Finished after {:?} clock cycles.", self.clk);
        }
        return !alocisw;
    }
}
