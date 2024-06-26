use crate::core::{Core, CoreStats};
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;
use crate::Optimizations;
use crate::{analyzer::Analyzable, bus::Bus};
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
    mp_bar: MultiProgress,
    rng: rand::rngs::mock::StepRng,
    optimizations: Optimizations,
}

impl System {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        record_streams: Vec<RecordStream>,
        show_process: bool,
        optimizations: Optimizations,
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
            mp_bar,
            rng: StepRng::new(0, 1),
            optimizations,
        }
    }

    /// Execute one simulator step (one cycle) of the multi core system.
    /// Returns true on end of simulation (all instructions executed).
    pub fn update(&mut self) -> bool {
        self.clk += 1;
        #[cfg(verbose)]
        println!("Step {:?}", self.clk);
        self.progress.inc(1);

        self.bus.update();
        self.active_cores.shuffle(&mut self.rng);

        // run 1: parse new instructions / update state
        let mut deactivated_cores: Vec<usize> = Vec::new();
        for core_id in &self.active_cores {
            if !self.cores[*core_id].step(&mut self.bus, self.clk) {
                deactivated_cores.push(*core_id);
            }
        }
        if !deactivated_cores.is_empty() {
            self.active_cores = self
                .active_cores
                .drain(..)
                .filter(|c| !deactivated_cores.contains(c))
                .collect();
        }

        // run 2: snoop other cores' actions
        for core in self.cores.iter_mut() {
            core.snoop(&mut self.bus);
        }

        // run 2.5: read broadcast optimization (if enabled)
        if self.optimizations.read_broadcast {
            for core in self.cores.iter_mut() {
                core.read_broadcast(&mut self.bus);
            }
        }

        // run 3: cleanup after bus snooping
        for core in self.cores.iter_mut() {
            core.after_snoop(&mut self.bus);
        }

        if self.active_cores.is_empty() {
            println!("Finished after {:?} clock cycles.", self.clk);
        }

        #[cfg(sanity_check)]
        self.sanity_check();
        self.active_cores.is_empty()
    }

    // compare cache state and cache protocol state
    #[cfg(sanity_check)]
    fn sanity_check(&self) {
        println!("Performing expensive sanity check.");
        for core in &self.cores {
            core.sanity_check();
        }
    }

    pub fn hide_progress(&self) {
        self.mp_bar.set_draw_target(ProgressDrawTarget::hidden());
    }
}

impl Analyzable for System {
    fn report(&self, stats: &mut crate::analyzer::Stats) {
        stats.exec_cycles = self.clk;
        (0..self.cores.len()).for_each(|_| stats.cores.push(CoreStats::default()));
        self.cores.iter().for_each(|c| c.report(stats));
        self.bus.report(stats);
    }
}
