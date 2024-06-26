mod analyzer;
mod bus;
mod cache;
mod core;
mod loader;
mod protocol;
mod record;
mod system;
mod utils;

pub use crate::analyzer::Analyzer;
pub use crate::bus::Bus;
pub use crate::core::Core;
pub use crate::loader::FileLoader;
pub use crate::protocol::ProtocolKind;
pub use crate::system::System;

#[derive(Debug, Default, Clone, Copy)]
pub struct Optimizations {
    pub read_broadcast: bool,
}
