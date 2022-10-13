use std::ops::Deref;

#[derive(Default)]
pub struct Bus {
    task: Option<Task>,
}

// MESI and Dragon bus actions combined
#[derive(Clone)]
pub enum BusAction {
    BusRdMem(u32),
    BusRdShared(u32),
    BusRdXMem(u32),
    BusRdXShared(u32),
    BusUpdMem(u32),
    BusUpdShared(u32),
    Flush(u32),
}

impl Deref for BusAction {
    type Target = u32;
    fn deref(&self) -> &u32 {
        match self {
            BusAction::BusRdMem(n) => n,
            BusAction::BusRdShared(n) => n,
            BusAction::BusRdXMem(n) => n,
            BusAction::BusRdXShared(n) => n,
            BusAction::BusUpdMem(n) => n,
            BusAction::BusUpdShared(n) => n,
            BusAction::Flush(n) => n,
        }
    }
}

impl std::fmt::Debug for BusAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusAction::BusRdMem(n) => write!(f, "BusRdMem(0x{:x})", n),
            BusAction::BusRdShared(n) => write!(f, "BusRdShared(0x{:x})", n),
            BusAction::BusRdXMem(n) => write!(f, "BusRdXMem(0x{:x})", n),
            BusAction::BusRdXShared(n) => write!(f, "BusRdXShared(0x{:x})", n),
            BusAction::BusUpdMem(n) => write!(f, "BusUpdMem(0x{:x})", n),
            BusAction::BusUpdShared(n) => write!(f, "BusUpdShared(0x{:x})", n),
            BusAction::Flush(n) => write!(f, "Flush(0x{:x})", n),
        }
    }
}

#[derive(Clone)]
pub struct Task {
    pub issuer_id: u32,
    pub remaining_cycles: u32,
    pub action: BusAction,
}

impl Bus {
    pub fn new() -> Self {
        Bus { task: None }
    }

    // pricelist
    pub fn price(action: &BusAction) -> u32 {
        match action {
            BusAction::BusRdMem(_) => 100,
            BusAction::BusRdShared(_) => 2,
            BusAction::BusRdXMem(_) => 100,
            BusAction::BusRdXShared(_) => 2,
            BusAction::BusUpdMem(_) => 100,
            BusAction::BusUpdShared(_) => 2,
            BusAction::Flush(_) => 100,
        }
    }

    pub fn put_on(&mut self, issuer_id: u32, action: BusAction) {
        assert!(self.task.is_none());
        self.task = Some(Task {
            issuer_id,
            remaining_cycles: Bus::price(&action),
            action,
        });
    }

    pub fn occupied(&self) -> bool {
        self.task.is_some()
    }

    pub fn update(&mut self) {
        if !self.occupied() {
            #[cfg(debug_assertions)]
            println!("Bus: empty");
            return;
        }
        let task = self.task.as_mut().unwrap();
        match task.remaining_cycles {
            0 => {
                self.task = None;

                #[cfg(debug_assertions)]
                println!("Bus: empty");
            }
            i => {
                task.remaining_cycles = i - 1;
                #[cfg(debug_assertions)]
                println!(
                    "Bus: {:?} by {:?}, remaining: {:?}",
                    self.task.as_ref().unwrap().action,
                    self.task.as_ref().unwrap().issuer_id,
                    self.task.as_ref().unwrap().remaining_cycles
                );
            }
        }
    }

    pub fn active_task(&mut self) -> Option<&mut Task> {
        self.task.as_mut()
    }
}
