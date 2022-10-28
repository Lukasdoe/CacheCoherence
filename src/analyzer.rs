use crate::cache::CacheStats;
use crate::core::CoreStats;

#[derive(Debug, Default)]
pub struct Stats {
    pub exec_cycles: usize,
    pub cores: Vec<CoreStats>,
    pub bus_traffic: usize,
    pub bus_num_invalid_or_upd: usize,
    pub cache: CacheStats,
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
             No. Data Cache Hits:               {:<10} ({:.2})\n\
             No. Data Cache Misses:             {:<10} ({:.2})\n\
             Bus Traffic:                       {:?} Bytes\n\
             No. Bus Invalidations or Updates:  {:?}\n\n\
             Core Statistics:\n",
            self.stats.exec_cycles,
            self.stats.cache.num_private_data_access,
            self.stats.cache.num_shared_data_access,
            self.stats.cache.num_data_cache_hits,
            (self.stats.cache.num_data_cache_hits as f64
                / self.stats.cores.iter().map(|c| c.mem_ops).sum::<usize>() as f64),
            self.stats.cache.num_data_cache_misses,
            (self.stats.cache.num_data_cache_misses as f64
                / self.stats.cores.iter().map(|c| c.mem_ops).sum::<usize>() as f64),
            self.stats.bus_traffic,
            self.stats.bus_num_invalid_or_upd
        ));
        for (id, core) in self.stats.cores.iter().enumerate() {
            s.push_str(&format!(
                "\tCore {:?} ({:?}):\n\
                 \t\tNo. Instructions:         {:?}\n\
                 \t\tExecution Cycles:         {:?}\n\
                 \t\tComputation Cycles:       {:<10} ({:.2})\n\
                 \t\tIdle Cycles:              {:<10} ({:.2})\n\
                 \t\tNo. Memory Instructions:  {:<10} ({:.2})\n\
                 \t\tNo. Load Instructions:    {:<10} ({:.2})\n\
                 \t\tNo. Store Insutrctions:   {:<10} ({:.2})\n\
                 \t\tNo. Data Cache Hits:      {:<10} ({:.2})\n\
                 \t\tNo. Data Cache Misses:    {:<10} ({:.2})\n\n",
                id,
                core.file_name,
                core.num_instructions,
                core.exec_cycles,
                core.compute_cycles,
                (core.compute_cycles as f64 / core.exec_cycles as f64),
                core.idle_cycles,
                (core.idle_cycles as f64 / core.exec_cycles as f64),
                core.mem_ops,
                (core.mem_ops as f64 / core.num_instructions as f64),
                core.load_instructions,
                (core.load_instructions as f64 / core.num_instructions as f64),
                core.store_instructions,
                (core.store_instructions as f64 / core.num_instructions as f64),
                core.cache.num_data_cache_hits,
                (core.cache.num_data_cache_hits as f64 / core.mem_ops as f64),
                core.cache.num_data_cache_misses,
                (core.cache.num_data_cache_misses as f64 / core.mem_ops as f64),
            ))
        }
        s
    }
}
