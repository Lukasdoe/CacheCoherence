#[macro_use]
extern crate lazy_static;

use logger::Logger;

lazy_static! {
    pub static ref LOGGER: Logger = Logger::new();
}

mod bus;
mod cache;
mod core;
mod loader;
mod protocol;
mod record;
mod system;
mod utils;

pub use crate::bus::Bus;
pub use crate::core::Core;
pub use crate::loader::FileLoader;
pub use crate::protocol::ProtocolKind;
pub use crate::system::System;
