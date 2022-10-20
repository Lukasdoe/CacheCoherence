use serde::{Deserialize, Serialize};

// MESI and Dragon bus actions combined
/// BusAction(address, size_in_bytes)
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum BusAction {
    BusRdMem(u32, usize),
    BusRdShared(u32, usize),
    BusRdXMem(u32, usize),
    BusRdXShared(u32, usize),
    BusUpdMem(u32, usize),
    BusUpdShared(u32, usize),
    Flush(u32, usize),
}

impl BusAction {
    pub fn extract_addr(action: BusAction) -> u32 {
        match action {
            BusAction::BusRdMem(n, _) => n,
            BusAction::BusRdShared(n, _) => n,
            BusAction::BusRdXMem(n, _) => n,
            BusAction::BusRdXShared(n, _) => n,
            BusAction::BusUpdMem(n, _) => n,
            BusAction::BusUpdShared(n, _) => n,
            BusAction::Flush(n, _) => n,
        }
    }

    pub fn extract_size(action: BusAction) -> usize {
        match action {
            BusAction::BusRdMem(_, c) => c,
            BusAction::BusRdShared(_, c) => c,
            BusAction::BusRdXMem(_, c) => c,
            BusAction::BusRdXShared(_, c) => c,
            BusAction::BusUpdMem(_, c) => c,
            BusAction::BusUpdShared(_, c) => c,
            BusAction::Flush(_, c) => c,
        }
    }
}

impl std::fmt::Debug for BusAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusAction::BusRdMem(n, c) => write!(f, "BusRdMem(0x{:x}, {:?})", n, c),
            BusAction::BusRdShared(n, c) => write!(f, "BusRdShared(0x{:x}, {:?})", n, c),
            BusAction::BusRdXMem(n, c) => write!(f, "BusRdXMem(0x{:x}, {:?})", n, c),
            BusAction::BusRdXShared(n, c) => write!(f, "BusRdXShared(0x{:x}, {:?})", n, c),
            BusAction::BusUpdMem(n, c) => write!(f, "BusUpdMem(0x{:x}, {:?})", n, c),
            BusAction::BusUpdShared(n, c) => write!(f, "BusUpdShared(0x{:x}, {:?})", n, c),
            BusAction::Flush(n, c) => write!(f, "Flush(0x{:x}, {:?})", n, c),
        }
    }
}
