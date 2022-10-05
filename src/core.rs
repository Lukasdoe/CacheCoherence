use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::{Label, RecordStream};
use crate::utils::Counter;
use crate::LOGGER;

pub struct Core {
    cache: Cache,
    alu: Counter,
    records: RecordStream,
    id: usize,
}

impl Core {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        records: RecordStream,
        id: usize,
    ) -> Self {
        println!("{:?} loaded into Core {:?}", records.file_name, id);
        LOGGER.lock().unwrap().log_core_init(&records.file_name, id);

        Core {
            cache: Cache::new(id, cache_size, associativity, block_size, protocol),
            alu: Counter::new(),
            records,
            id,
        }
    }

    /// Simulate one cycle. Return false if no more instructions are left to process.
    pub fn step(&mut self, bus: &mut Bus) -> bool {
        // stall, if required. Remember: if they return false, then they didn't work yet.
        if self.alu.update() || self.cache.update() {
            LOGGER
                .lock()
                .unwrap()
                .log_core_state(self.id, None, self.alu.value);
            return true;
        }

        if let Some(record) = self.records.next() {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Processing: {:?} {:#x}",
                self.id, record.label, record.value
            );
            match (&record.label, record.value) {
                (Label::Load, ref value) => self.cache.load(*value),
                (Label::Store, ref value) => self.cache.store(*value),
                (Label::Other, ref value) => self.alu.value = *value,
            }
            // they still have a free step in this cycle!
            self.alu.update();
            self.cache.update();

            // TODO: should this be after the update?
            LOGGER.lock().unwrap().log_core_state(
                self.id,
                Some(format!("{:?} {:?}", record.label, record.value)),
                self.alu.value,
            );
            true
        } else {
            false
        }
    }
}
