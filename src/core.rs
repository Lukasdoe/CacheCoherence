use crate::alu::Alu;
use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::{Label, Record, RecordStream};

pub struct Core {
    cache: Cache,
    alu: Alu,
    records: RecordStream,
}

impl Core {
    pub fn new(
        protocol: &ProtocolKind,
        capacity: usize,
        associativity: usize,
        block_size: usize,
        records: RecordStream,
    ) -> Self {
        println!("{:?}", records.file_name);

        Core {
            cache: Cache::new(capacity, associativity, block_size, protocol),
            alu: Alu::new(),
            records,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if self.alu.update() {
            return;
        }

        if let Some(record) = self.records.next() {
            println!("{:?} {:?}", record.label, record.value);
            match (record.label, record.value) {
                (Label::Load, value) => (),
                (Label::Store, value) => (),
                (Label::Other, value) => self.alu.set(value as isize),
            }
        }
    }
}
