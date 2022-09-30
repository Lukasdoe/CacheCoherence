use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;

pub struct Core {
    cache: Cache,
    records: RecordStream,
}

impl Core {
    pub fn new(kind: &ProtocolKind, capacity: usize, records: RecordStream) -> Self {
        println!("{:?}", records.file_name);

        Core {
            cache: Cache::new(capacity, kind),
            records,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if let Some(t) = self.records.next() {
            println!("{:?} {:?}", self.records.file_name, t);
        }
    }
}
