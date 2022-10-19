use logger::{LogEntry, Step};

use crate::bus::Bus;
use crate::core::Core;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;
use crate::LOGGER;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::{rngs::mock::StepRng, seq::SliceRandom};

/// word size in bytes
pub const WORD_SIZE: usize = 4;

pub struct System {
    cores: Vec<Core>,
    active_cores: Vec<usize>,
    bus: Bus,
    clk: usize,
    progress: ProgressBar,
    rng: rand::rngs::mock::StepRng,
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
            active_cores: (0..cores.len()).collect(),
            cores,
            bus: Bus::new(),
            clk: 0,
            progress: system_progress,
            rng: StepRng::new(0, 1),
        }
    }

    /// Execute one simulator step (one cycle) of the multi core system.
    /// Returns true on end of simulation (all instructions executed).
    pub fn update(&mut self) -> bool {
        self.clk += 1;
        #[cfg(debug_assertions)]
        println!("Step {:?}", self.clk);
        self.progress.inc(1);
        LOGGER.write(LogEntry::Step(Step { clk: self.clk }));

        self.bus.update();
        self.active_cores.shuffle(&mut self.rng);

        // run 1: parse new instructions / update state
        self.active_cores = self
            .active_cores
            .drain(..)
            .filter(|core_id| self.cores[*core_id].step(&mut self.bus))
            .collect();

        // run 2: snoop other cores' actions
        for core in self.cores.iter_mut() {
            core.snoop(&mut self.bus);
        }

        // run 3: cleanup after bus snooping
        for core in self.cores.iter_mut() {
            core.after_snoop(&mut self.bus);
        }

        if self.active_cores.is_empty() {
            println!("Finished after {:?} clock cycles.", self.clk);
        }
        self.active_cores.is_empty()
    }

    // TODO: Sanity check
}
