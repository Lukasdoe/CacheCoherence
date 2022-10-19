use logger::*;
use shared::bus::BusAction;
use shared::record::Label;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AnalyzerError {
    m: String,
}

impl Error for AnalyzerError {}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.m, f)
    }
}

impl AnalyzerError {
    pub fn new(m: &str) -> AnalyzerError {
        AnalyzerError { m: String::from(m) }
    }
}

#[derive(Debug)]
pub struct Stats {
    pub exec_cycles: usize,
    pub cores: Vec<CoreStats>,
    pub bus_traffic: usize,
    pub bus_num_invalid_or_upd: usize,
    pub num_private_data_access: usize,
    pub num_shared_data_access: usize,
}

#[derive(Default, Clone, Debug)]
pub struct CoreStats {
    pub file_name: String,
    pub exec_cycles: usize,
    pub compute_cycles: usize,
    pub mem_ops: usize,
    pub idle_cycles: usize,
    pub num_data_cache_misses: usize,
    pub num_data_cache_hits: usize,
}

pub struct Analyzer {}

impl Analyzer {
    pub fn extract_stats(logger: &mut RLogger) -> Result<Stats, AnalyzerError> {
        let mut bus_traffic = 0;
        let mut bus_num_invalid_or_upd = 0;
        let mut num_private_data_access = 0;
        let mut num_shared_data_access = 0;

        let sys_info = match logger.read() {
            Some(LogEntry::InitSystem(s)) => s,
            Some(e) => {
                return Err(AnalyzerError::new(&format!(
                    "Invalid first log entry, expected InitSystem but found {:?}.",
                    e
                )))
            }
            None => return Err(AnalyzerError::new("Logfile does not contain any entries.")),
        };
        let mut cores = vec![CoreStats::default(); sys_info.num_cores];

        let mut cur_step = 0;
        let mut per_step_accesssssss_debug = 0;
        for entry in std::iter::from_fn(|| logger.read()) {
            match entry {
                LogEntry::InitCore(InitCore { id, file_name }) => cores[id].file_name = file_name,
                LogEntry::Step(Step { clk }) => {
                    cur_step = clk;
                    per_step_accesssssss_debug = 0
                }
                LogEntry::InstrFetch(InstrFetch { id, type_, arg }) => {
                    match type_ {
                        Label::Load | Label::Store => cores[id].mem_ops += 1,
                        Label::Other => cores[id].compute_cycles += arg as usize,
                    };
                }
                LogEntry::CoreHalt(CoreHalt { id }) => cores[id].exec_cycles = cur_step,
                LogEntry::CoreStallMemory(CoreStallMemory { id }) => cores[id].idle_cycles += 1,
                LogEntry::CacheMiss(CacheMiss { id, .. }) => cores[id].num_data_cache_misses += 1,
                LogEntry::CacheHit(CacheHit { id, .. }) => cores[id].num_data_cache_hits += 1,
                LogEntry::BusFinish(BusFinish { action, .. }) => {
                    match action {
                        BusAction::BusUpdMem(_, _)
                        | BusAction::BusUpdShared(_, _)
                        | BusAction::BusRdXMem(_, _)
                        | BusAction::BusRdXShared(_, _) => bus_num_invalid_or_upd += 1,
                        _ => (),
                    }
                    bus_traffic += BusAction::extract_size(action)
                }
                LogEntry::CachePrivateAccess(_) => {
                    num_private_data_access += 1;
                    per_step_accesssssss_debug += 1
                }
                LogEntry::CacheSharedAccess(_) => {
                    num_shared_data_access += 1;
                    per_step_accesssssss_debug += 1
                }
                LogEntry::BusStart(_) => (),
                LogEntry::BusClear(_) => (),
                LogEntry::InitSystem(_) => (),
                LogEntry::CoreStallALU(_) => (),
            }
            assert!(per_step_accesssssss_debug < 4);
        }
        Ok(Stats {
            exec_cycles: cur_step,
            cores,
            bus_traffic,
            bus_num_invalid_or_upd,
            num_private_data_access,
            num_shared_data_access,
        })
    }

    pub fn pretty_print(stats: &Stats) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "# Execution Cycles (total):    {:?}\n\
             # Private Data Accesses:       {:?}\n\
             # Shared Data Accesses:        {:?}\n\
             Bus Traffic:                   {:?}\n\
             Bus Invalidations or Updates:  {:?}\n
             \
             Core Statistics:\n",
            stats.exec_cycles,
            stats.num_private_data_access,
            stats.num_shared_data_access,
            stats.bus_traffic,
            stats.bus_num_invalid_or_upd
        ));
        for (id, core) in stats.cores.iter().enumerate() {
            s.push_str(&format!(
                "\tCore {:?} ({:?}):\n\
                 \t\tExecution Cycles:     {:?}\n\
                 \t\tComputation Cycles:   {:?}\n\
                 \t\tIdle Cycles:          {:?}\n\
                 \t\t# Memory Operations:  {:?}\n\
                 \t\t# Data Cache Hits:    {:?}\n\
                 \t\t# Data Cache Misses:  {:?}\n\n",
                id,
                core.file_name,
                core.exec_cycles,
                core.compute_cycles,
                core.idle_cycles,
                core.mem_ops,
                core.num_data_cache_hits,
                core.num_data_cache_misses,
            ))
        }
        s
    }
}
