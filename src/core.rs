use crate::alu::Alu;
use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::{Label, Record, RecordStream};

pub struct Core {
    cache: Cache,
    alu: Alu,
    records: RecordStream,

    debug_id: usize,
}

impl Core {
    pub fn new(
        protocol: &ProtocolKind,
        capacity: usize,
        associativity: usize,
        block_size: usize,
        records: RecordStream,
        debug_id: usize,
    ) -> Self {
        println!("{:?} loaded into Core {:?}", records.file_name, debug_id);

        Core {
            cache: Cache::new(capacity, associativity, block_size, protocol),
            alu: Alu::new(),
            records,
            debug_id,
        }
    }

    /// Simulate one cycle. Return false if no more instructions are left to process.
    pub fn step(&mut self, bus: &mut Bus) -> bool {
        // stall, if required. Remember: if they return false, then they didn't work yet.
        if self.alu.update() || self.cache.update() {
            return true;
        }

        if let Some(record) = self.records.next() {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Processing: {:?} {:#x}",
                self.debug_id, record.label, record.value
            );
            match (record.label, record.value) {
                (Label::Load, value) => self.cache.load(value),
                (Label::Store, value) => self.cache.store(value),
                (Label::Other, value) => self.alu.set(value as isize),
            }
            // they still have a free step in this cycle!
            self.alu.update();
            self.cache.update();
            true
        } else {
            false
        }
    }
}
