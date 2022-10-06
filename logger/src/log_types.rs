use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum LogEntry {
    EnvInfo(EnvInfo),
    Step(Step),
    CoreInit(CoreInit),
    CoreState(CoreState),
    CacheState(CacheState),
    CacheAccess(CacheAccess),
    CacheUpdate(CacheUpdate),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CoreInit {
    pub file_name: String,
    pub id: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct EnvInfo {
    pub protocol: String,
    pub cache_size: usize,
    pub associativity: usize,
    pub block_size: usize,
    pub num_cores: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Step {
    pub clk: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CoreState {
    pub id: usize,
    pub record: Option<String>,
    pub alu_cnt: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheState {
    pub core_id: usize,
    pub cnt: u32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheAccess {
    pub core_id: usize,

    /// hit = true, miss = false
    pub hit_or_miss: bool,
    pub tag: u32,
    pub index: usize,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct CacheUpdate {
    pub core_id: usize,

    pub old_tag: Option<u32>,
    pub new_tag: u32,
    pub index: usize,
    pub block: usize,
}
