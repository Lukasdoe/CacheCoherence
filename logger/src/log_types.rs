use serde::{Deserialize, Serialize};
use shared::bus::BusAction;
use shared::record;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum LogEntry {
    InitSystem(InitSystem),
    InitCore(InitCore),
    Step(Step),
    InstrFetch(InstrFetch),
    CoreHalt(CoreHalt),
    CoreStallALU(CoreStallALU),
    CoreStallMemory(CoreStallMemory),
    CacheMiss(CacheMiss),
    CacheHit(CacheHit),
    CachePrivateAccess(CachePrivateAccess),
    CacheSharedAccess(CacheSharedAccess),
    BusFinish(BusFinish),
    BusStart(BusStart),
    BusClear(BusClear),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InitSystem {
    pub protocol: String,
    pub cache_size: usize,
    pub associativity: usize,
    pub block_size: usize,
    pub num_cores: usize,
    pub archive_name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InitCore {
    pub id: usize,
    pub file_name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Step {
    pub clk: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InstrFetch {
    pub id: usize,
    pub type_: record::Label,
    pub arg: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CoreHalt {
    pub id: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CoreStallALU {
    pub id: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CoreStallMemory {
    pub id: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheMiss {
    pub id: usize,
    pub addr: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheHit {
    pub id: usize,
    pub addr: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct BusFinish {
    pub issuer_id: usize,
    pub action: BusAction,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct BusStart {
    pub issuer_id: usize,
    pub action: BusAction,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct BusClear {}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CachePrivateAccess {
    pub id: usize,
    pub addr: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheSharedAccess {
    pub id: usize,
    pub addr: u32,
}
