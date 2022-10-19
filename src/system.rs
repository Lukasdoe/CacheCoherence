use crate::bus::Bus;
use crate::core::Core;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

pub struct System {
    cores: Vec<Core>,
    bus: Bus,
    clk: usize,
    progress: ProgressBar,
}

impl System {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
        show_process: bool,
    ) -> Self {
        let mp_bar = MultiProgress::new();
        let system_progress = mp_bar
            .add(ProgressBar::new_spinner())
            .with_prefix("System Clock")
            .with_style(
                ProgressStyle::with_template("{prefix:.bold.dim} {human_pos:>10} Cycles").unwrap(),
            );
        let cores: Vec<Core> = record_streams
            .into_iter()
            .enumerate()
            .map(|(id, stream)| {
                Core::new(
                    protocol,
                    cache_size,
                    associativity,
                    block_size,
                    stream,
                    id,
                    &mp_bar,
                )
            })
            .collect();

        if !show_process {
            mp_bar.set_draw_target(ProgressDrawTarget::hidden());
        }
        System {
            cores,
            bus: Bus::new(),
            clk: 0,
            progress: system_progress,
        }
    }

    /// Execute one simulator step (one cycle) of the multi core system.
    /// Returns true on end of simulation (all instructions executed).
    pub fn update(&mut self) -> bool {
        self.clk += 1;
        #[cfg(debug_assertions)]
        println!("Step {:?}", self.clk);
        self.progress.inc(1);

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
        !alocisw
    }

    // TODO: Sanity check
}
