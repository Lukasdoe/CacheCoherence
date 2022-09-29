use clap::ArgEnum;

pub mod dragon;
pub mod mesi;

pub trait Protocol {}

#[derive(Clone, Debug, ArgEnum)]
pub enum ProtocolKind {
    Mesi,
    Dragon,
}

pub struct ProtocolBuilder;

impl ProtocolBuilder {
    pub fn new(kind: &ProtocolKind) -> Box<dyn Protocol> {
        match kind {
            ProtocolKind::Dragon => Box::new(dragon::Dragon::new()),
            ProtocolKind::Mesi => Box::new(mesi::Mesi::new()),
        }
    }
}
