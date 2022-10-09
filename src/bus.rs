use std::ops::Deref;

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

    pub fn check(task_id: u32) {
        // 1. Loop: Update bus, state and everything
        // 2. Loop: Snooping only, like S' vs S and S -> I
        // 3. Loop: Update state based on snooping results (dragon only)
    }

    pub fn put_on(&mut self, issuer_id: u32, action: BusAction) {
        assert!(self.task.is_none());
        self.task = Some(Task {
            issuer_id,
            remaining_cycles: Bus::price(&action),
            action: action,
        });
    }

    pub fn occupied(&self) -> bool {
        self.task.is_some()
    }

    pub fn active_task(&mut self) -> Option<&mut Task> {
        self.task.as_mut()
    }
}
