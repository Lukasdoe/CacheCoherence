#[derive(Default)]
pub struct Bus {
    // task: Option<Task>,
}

// MESI and Dragon bus actions combined
// TODO: Dragon protocol requires differences for BusRd, whether being from other cache or memory
pub enum BusAction {
    BusRd,
    BusRdX,
    BusUpd,
    Flush,
}

pub struct Task {
    task_id: u32,
    remaining_time: u32,
}

pub struct Transaction {
    addr: usize,
    action: BusAction,
}

impl Bus {
    pub fn new() -> Self {
        Bus {}
    }

    pub fn check(task_id: u32) {

        // 1. Loop: Update bus, state and everything
        // 2. Loop: Snooping only, like S' vs S and S -> I
        // 3. Loop: Update state based on snooping results (dragon only)
    }

    pub fn occupied(&self) -> bool {
        // self.task.is_some()
        return false;
    }
}
