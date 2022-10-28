use cacher::{Analyzer, FileLoader, ProtocolKind, System};

struct ProgramArgs {
    /// Cache coherence protocol
    protocol: ProtocolKind,

    /// Path to the benchmark archive, e.g. "./blackscholes_four.zip"
    input_file: String,

    /// Cache size in bytes
    cache_size: usize,

    /// Cache associativity
    associativity: usize,

    /// Cache block size in bytes
    block_size: usize,

    /// Disable progress display
    no_progress: bool,
}

impl ProgramArgs {
    fn new(
        input_file: String,
        protocol: ProtocolKind,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        no_progress: bool,
    ) -> Self {
        ProgramArgs {
            protocol,
            input_file,
            cache_size,
            associativity,
            block_size,
            no_progress,
        }
    }
}

fn run(args: ProgramArgs) -> Analyzer {
    let record_streams = match FileLoader::open(&args.input_file, !args.no_progress) {
        Ok(streams) => streams,
        Err(e) => {
            println!(
                "Error during loading of the supplied input file: {:?}",
                e.to_string()
            );
            std::process::exit(e.raw_os_error().unwrap_or(1));
        }
    };

    let mut system = System::new(
        &args.protocol,
        args.cache_size,
        args.associativity,
        args.block_size,
        record_streams,
        !args.no_progress,
    );

    loop {
        if system.update() {
            break;
        }
    }

    let mut analyzer = Analyzer::new();
    analyzer.digest(system);

    analyzer
}

#[test]
fn read_miss() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/read_miss.zip"),
        ProtocolKind::Dragon,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(analyzer.stats.exec_cycles, 102);
    assert_eq!(analyzer.stats.bus_traffic, 4);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 1);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 1);
}

#[test]
fn read_hit() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/read_hit.zip"),
        ProtocolKind::Dragon,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(analyzer.stats.exec_cycles, 103);
    assert_eq!(analyzer.stats.bus_traffic, 4);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 2);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 1);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 1);
}

#[test]
fn write_miss() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/write_miss.zip"),
        ProtocolKind::Dragon,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(analyzer.stats.exec_cycles, 104);
    assert_eq!(analyzer.stats.bus_traffic, 4);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 1);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 1);
}

#[test]
fn write_hit() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/write_hit.zip"),
        ProtocolKind::Dragon,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(analyzer.stats.exec_cycles, 105);
    assert_eq!(analyzer.stats.bus_traffic, 4);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 1);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 1);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 1);
}

#[test]
fn evict() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/evict.zip"),
        ProtocolKind::Dragon,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(analyzer.stats.exec_cycles, 304);
    assert_eq!(analyzer.stats.bus_traffic, 12);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 3);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 3);
}

#[test]
fn sequence_16_1_4() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/sequence.zip"),
        ProtocolKind::Mesi,
        16,
        1,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(
        analyzer.stats.exec_cycles,
        1 + 101 + 10 + 102 + 101 + 101 + 1 + 101 + 101 + 101 + 102 + 101 + 101 + 102 + 3
    );
    assert_eq!(analyzer.stats.bus_traffic, 4 * 11);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 8);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 1);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 8);
}

#[test]
fn sequence_16_2_4() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/sequence.zip"),
        ProtocolKind::Dragon,
        16,
        2,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(
        analyzer.stats.exec_cycles,
        1 + 101 + 10 + 102 + 1 + 1 + 101 + 101 + 101 + 101 + 102 + 101 + 102 + 3
    );
    assert_eq!(analyzer.stats.bus_traffic, 4 * 9);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 8);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 2);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 7);
}

#[test]
fn sequence_16_4_4() {
    let args = ProgramArgs::new(
        String::from("data/single_thread/sequence.zip"),
        ProtocolKind::Dragon,
        16,
        4,
        4,
        true,
    );

    let analyzer = run(args);
    assert_eq!(
        analyzer.stats.exec_cycles,
        1 + 101 + 10 + 102 + 1 + 1 + 101 + 1 + 101 + 101 + 102 + 1
    );
    assert_eq!(analyzer.stats.bus_traffic, 4 * 6);
    assert_eq!(analyzer.stats.bus_num_invalid_or_upd, 0);
    assert_eq!(analyzer.stats.cache.num_private_data_access, 6);
    assert_eq!(analyzer.stats.cache.num_shared_data_access, 0);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_hits, 4);
    assert_eq!(analyzer.stats.cores[0].cache.num_data_cache_misses, 5);
}
