use serde::{Deserialize, Serialize};

use crate::alu::Alu;
use crate::bus::Bus;
use crate::cache::{Cache, CacheState};
use crate::protocol::ProtocolKind;
use crate::record::{Label, Record, RecordStream};

#[derive(Default, Serialize)]
pub struct CoreState {
    id: usize,
    record: Option<Record>,
    alu: u32,
}

pub struct Core {
    cache: Cache,
    alu: Alu,
    records: RecordStream,
    debug_id: usize,
    state: CoreState,
}

impl Core {
    pub fn new(
        protocol: &ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        records: RecordStream,
        debug_id: usize,
    ) -> Self {
        println!("{:?} loaded into Core {:?}", records.file_name, debug_id);
        let mut state = CoreState::default();
        state.id(debug_id);

        Core {
            cache: Cache::new(cache_size, associativity, block_size, protocol),
            alu: Alu::new(),
            records,
            debug_id,
            state,
        }
    }

    /// Simulate one cycle. Return false if no more instructions are left to process.
    pub fn step(&mut self, bus: &mut Bus) -> bool {
        // stall, if required. Remember: if they return false, then they didn't work yet.
        self.state
            .id(self.debug_id)
            .record(None)
            .alu(self.alu.get());

        if self.alu.update() || self.cache.update() {
            return true;
        }

        if let Some(record) = self.records.next() {
            #[cfg(debug_assertions)]
            println!(
                "({:?}) Processing: {:?} {:#x}",
                self.debug_id, record.label, record.value
            );
            match (record.label, record.value) {
                (Label::Load, value) => self.cache.load(value),
                (Label::Store, value) => self.cache.store(value),
                (Label::Other, value) => self.alu.set(value),
            }
            // they still have a free step in this cycle!
            self.state.record(Some(record)).alu(self.alu.get());
            self.alu.update();
            self.cache.update();
            true
        } else {
            false
        }
    }

    pub fn state(&self) -> (&CoreState, &CacheState) {
        (&self.state, self.cache.state())
    }
}

impl CoreState {
    pub fn id(&mut self, value: usize) -> &mut Self {
        self.id = value;
        self
    }

    pub fn record(&mut self, value: Option<Record>) -> &mut Self {
        self.record = value;
        self
    }

    pub fn alu(&mut self, value: u32) -> &mut Self {
        self.alu = value;
        self
    }
}
