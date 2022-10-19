use logger::*;
use shared::bus::BusAction;

use crate::LOGGER;

#[derive(Default, Debug)]
pub struct Bus {
    task: Option<Task>,
}

#[derive(Clone, Copy, Debug)]
pub struct Task {
    pub issuer_id: usize,
    pub remaining_cycles: usize,
    pub action: BusAction,
}

impl Bus {
    pub fn new() -> Self {
        Bus { task: None }
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
        LOGGER.write(LogEntry::BusFinish(BusFinish { issuer_id, action }));

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

    /// Get currently scheduled bus transaction (if any)
    pub fn active_task(&mut self) -> Option<&mut Task> {
        self.task.as_mut()
    }
}
