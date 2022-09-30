use crate::bus::Bus;
use crate::cache::Cache;
use crate::protocol::ProtocolKind;
use crate::record::RecordStream;

pub struct Core {
    cache: Cache,
    stream: RecordStream,
}

impl Core {
    pub fn new(kind: &ProtocolKind, capacity: usize, mut stream: RecordStream) -> Self {
        println!("{:?}", stream.file_name);
        println!("{:?}", stream.start().last().unwrap());
        println!("{:?}", stream.start().count());

        Core {
            cache: Cache::new(capacity, kind),
            stream,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {}
}
