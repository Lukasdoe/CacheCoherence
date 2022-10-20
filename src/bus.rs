use crate::analyzer::Analyzable;

// MESI and Dragon bus actions combined
/// BusAction(address, size_in_bytes)
#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Default, Debug)]
pub struct Bus {
    task: Option<Task>,
    stats: BusStats,
}

#[derive(Default, Debug)]
struct BusStats {
    pub traffic: usize,
    pub num_invalid_or_upd: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Task {
    pub issuer_id: usize,
    pub remaining_cycles: usize,
    pub action: BusAction,
}

impl Bus {
    pub fn new() -> Self {
        Bus::default()
    }

    /// Query number of cycles required for the entered action
    pub fn price(action: &BusAction) -> usize {
        match action {
            BusAction::BusRdMem(_, _) => 100,
            BusAction::BusRdShared(_, c) => 2 * (c / 4),
            BusAction::BusRdXMem(_, _) => 100,
            BusAction::BusRdXShared(_, c) => 2 * (c / 4),
            BusAction::BusUpdMem(_, _) => 100,
            BusAction::BusUpdShared(_, c) => 2 * (c / 4),
            BusAction::Flush(_, _) => 100,
        }
    }

    /// Schedule bus transaction
    pub fn put_on(&mut self, issuer_id: usize, action: BusAction) {
        assert!(self.task.is_none());

        match action {
            BusAction::BusUpdMem(_, _)
            | BusAction::BusUpdShared(_, _)
            | BusAction::BusRdXMem(_, _)
            | BusAction::BusRdXShared(_, _) => self.stats.num_invalid_or_upd += 1,
            _ => (),
        }
        self.stats.traffic += BusAction::extract_size(action);

        self.task = Some(Task {
            issuer_id,
            remaining_cycles: Bus::price(&action),
            action,
        });
    }

    /// Clear current bus transaction
    pub fn clear(&mut self) {
        self.task = None;
    }

    /// Returns true if the bus is currently busy
    pub fn occupied(&self) -> bool {
        self.task.is_some()
    }

    /// Advance current bus transaction by one cycle (if any)
    pub fn update(&mut self) {
        if !self.occupied() {
            #[cfg(verbose)]
            println!("Bus: empty");
            return;
        }
        let task = self.task.as_mut().unwrap();
        match task.remaining_cycles {
            0 => {
                self.task = None;

                #[cfg(verbose)]
                println!("Bus: empty");
            }
            i => {
                task.remaining_cycles = i - 1;
                #[cfg(verbose)]
                println!(
                    "Bus: {:?} by {:?}, remaining: {:?}",
                    self.task.as_ref().unwrap().action,
                    self.task.as_ref().unwrap().issuer_id,
                    self.task.as_ref().unwrap().remaining_cycles
                );
            }
        }
    }

    /// Get currently scheduled bus transaction (if any)
    pub fn active_task(&mut self) -> Option<&mut Task> {
        self.task.as_mut()
    }
}

impl Analyzable for Bus {
    fn report(&self, stats: &mut crate::analyzer::Stats) {
        stats.bus_traffic = self.stats.traffic;
        stats.bus_num_invalid_or_upd = self.stats.num_invalid_or_upd;
    }
}
