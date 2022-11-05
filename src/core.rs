use crate::analyzer::Analyzable;
use crate::bus::Bus;
use crate::cache::{Cache, CacheStats};
use crate::protocol::ProtocolKind;
use crate::record::{Label, RecordStream};
use crate::utils::Counter;
use indicatif::*;

#[derive(Default, Clone, Debug)]
pub struct CoreStats {
    pub file_name: String,
    pub exec_cycles: usize,
    pub compute_cycles: usize,
    pub mem_ops: usize,
    pub idle_cycles: usize,
    pub num_instructions: usize,
    pub load_instructions: usize,
    pub store_instructions: usize,
    pub cache: CacheStats,
}

pub struct Core {
    cache: Cache,
    alu: Counter,
    records: RecordStream,
    id: usize,
    progress_bar: ProgressBar,
    stats: CoreStats,
}

impl Core {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        records: RecordStream,
        id: usize,
        mp_bar: &MultiProgress,
    ) -> Self {
        println!("({:?}) loaded {:?}", id, records.file_name);

        let pb = mp_bar
            .add(ProgressBar::new(records.line_count as u64))
            .with_prefix(format!("Core {:?}", id));
        pb.set_style(
            ProgressStyle::with_template(
                "{prefix:.bold.dim} [{wide_bar:.cyan/blue}] {human_pos:>10} / {human_len:>10} ({percent:>3}%)",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        Core {
            cache: Cache::new(id, cache_size, associativity, block_size, protocol),
            alu: Counter::new(),
            progress_bar: pb,
            id,
            stats: CoreStats {
                file_name: records.file_name.clone(),
                ..CoreStats::default()
            },
            records,
        }
    }

    /// Simulate one cycle. Return false if no more instructions are left to process.
    pub fn step(&mut self, bus: &mut Bus, clk: usize) -> bool {
        // stall, if required. Remember: if they return false, then they didn't work yet.
        if self.alu.update() {
            return true;
        }
        if self.cache.update(bus) {
            self.stats.idle_cycles += 1;
            return true;
        }

        if let Some(record) = self.records.next() {
            #[cfg(verbose)]
            println!(
                "({:?}) Processing new: {:?} {:#x}",
                self.id, record.label, record.value
            );
            match record.label {
                Label::Load => {
                    self.stats.mem_ops += 1;
                    self.stats.load_instructions += 1
                }
                Label::Store => {
                    self.stats.mem_ops += 1;
                    self.stats.store_instructions += 1
                }
                Label::Other => self.stats.compute_cycles += record.value as usize,
            };

            self.stats.num_instructions += 1;

            self.progress_bar.inc(1);

            match (&record.label, record.value) {
                (Label::Load, ref value) => self.cache.load(*value),
                (Label::Store, ref value) => self.cache.store(*value),
                (Label::Other, ref value) => self.alu.value = *value,
            }
            // they still have a free step in this cycle!
            self.alu.update();
            self.cache.update(bus);
            true
        } else {
            self.stats.exec_cycles = clk;
            self.progress_bar.finish();
            false
        }
    }

    pub fn snoop(&mut self, bus: &mut Bus) {
        self.cache.snoop(bus);
    }

    pub fn after_snoop(&mut self, bus: &mut Bus) {
        self.cache.after_snoop(bus);
    }

    #[cfg(sanity_check)]
    pub fn sanity_check(&self) {
        self.cache.sanity_check();
    }

    pub fn read_broadcast(&mut self, bus: &mut Bus) {
        self.cache.read_broadcast(bus);
    }
}

impl Analyzable for Core {
    fn report(&self, stats: &mut crate::analyzer::Stats) {
        let c_stats = &mut stats.cores[self.id];
        c_stats.file_name = self.stats.file_name.clone();
        c_stats.exec_cycles = self.stats.exec_cycles;

        c_stats.compute_cycles = self.stats.compute_cycles;
        c_stats.mem_ops = self.stats.mem_ops;
        c_stats.idle_cycles = self.stats.idle_cycles;

        c_stats.num_instructions = self.stats.num_instructions;
        c_stats.load_instructions = self.stats.load_instructions;
        c_stats.store_instructions = self.stats.store_instructions;

        self.cache.report(stats);
    }
}
