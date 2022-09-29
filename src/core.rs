use crate::protocol::{Protocol, ProtocolBuilder, ProtocolKind};

pub struct Core {
    protocol: Box<dyn Protocol>,
}

impl Core {
    pub fn new(kind: &ProtocolKind) -> Self {
        Core {
            protocol: ProtocolBuilder::new(kind),
        }
    }
}
