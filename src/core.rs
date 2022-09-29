use crate::cache::Cache;
use crate::protocol::ProtocolKind;

pub struct Core {
    cache: Cache,
}

impl Core {
    pub fn new(kind: &ProtocolKind, capacity: usize) -> Self {
        Core {
            cache: Cache::new(capacity, kind),
        }
    }
}
