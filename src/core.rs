use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::{Label, RecordStream};
use crate::utils::Counter;
use indicatif::*;

pub struct Core {
    cache: Cache,
    alu: Counter,
    records: RecordStream,
    id: usize,
    progress_bar: ProgressBar,
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
                "{prefix:.bold.dim} [{wide_bar:.cyan/blue}] {human_pos:>10} / {human_len:>10} ({percent:>3})",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        Core {
            cache: Cache::new(id, cache_size, associativity, block_size, protocol),
            alu: Counter::new(),
            progress_bar: pb,
            records,
            id,
        }
    }

    /// Simulate one cycle. Return false if no more instructions are left to process.
    pub fn step(&mut self, bus: &mut Bus) -> bool {
        // stall, if required. Remember: if they return false, then they didn't work yet.
        if self.alu.update() || self.cache.update(bus) {
            return true;
        }

        if let Some(record) = self.records.next() {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Processing new: {:?} {:#x}",
                self.id, record.label, record.value
            );
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
}
