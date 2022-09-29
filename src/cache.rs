use crate::protocol::{Protocol, ProtocolBuilder, ProtocolKind};

pub struct Cache {
    cache: Vec<usize>,
    cache_state: Vec<usize>,
    protocol: Box<dyn Protocol>,
}

impl Cache {
    pub fn new(capacity: usize, kind: &ProtocolKind) -> Self {
        Cache {
            cache: Vec::with_capacity(capacity),
            cache_state: Vec::with_capacity(capacity),
            protocol: ProtocolBuilder::new(kind),
        }
    }
}
