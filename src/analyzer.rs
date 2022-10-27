#[derive(Debug, Default)]
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

pub trait Analyzable {
    fn report(&self, stats: &mut Stats);
}

#[derive(Default, Debug)]
pub struct Analyzer {
    pub stats: Stats,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer::default()
    }

    pub fn digest<A: Analyzable>(&mut self, logger: A) {
        logger.report(&mut self.stats);
    }

    pub fn pretty_print(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "No. Execution Cycles (total):      {:?}\n\
             No. Private Data Accesses:         {:?}\n\
             No. Shared Data Accesses:          {:?}\n\
             Bus Traffic:                       {:?} Bytes\n\
             No. Bus Invalidations or Updates:  {:?}\n
             \
             Core Statistics:\n",
            self.stats.exec_cycles,
            self.stats.num_private_data_access,
            self.stats.num_shared_data_access,
            self.stats.bus_traffic,
            self.stats.bus_num_invalid_or_upd
        ));
        for (id, core) in self.stats.cores.iter().enumerate() {
            s.push_str(&format!(
                "\tCore {:?} ({:?}):\n\
                 \t\tExecution Cycles:       {:?}\n\
                 \t\tComputation Cycles:     {:?}\n\
                 \t\tIdle Cycles:            {:?}\n\
                 \t\tNo. Memory Operations:  {:?}\n\
                 \t\tNo. Data Cache Hits:    {:?}\n\
                 \t\tNo. Data Cache Misses:  {:?}\n\n",
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
