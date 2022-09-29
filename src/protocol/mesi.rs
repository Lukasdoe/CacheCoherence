use super::Protocol;

pub struct Mesi;

impl Mesi {
    pub fn new() -> Self {
        Mesi {}
    }
}

impl Protocol for Mesi {}
