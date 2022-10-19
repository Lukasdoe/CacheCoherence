use cacher::LOGGER;
use cacher::{FileLoader, ProtocolKind, System};
use clap::Parser;
use logger::{InitSystem, LogEntry};

#[derive(Parser, Debug)]
#[clap(version,
    about = "\x1b[1mCACHER\x1b[0m - \x1b[1mCA\x1b[0mche \x1b[1mC\x1b[0mo\x1b[1mH\x1b[0merence \x1b[1mE\x1b[0mmulato\x1b[1mR\x1b[0m",
    long_about = None)]
struct ProgramArgs {
    /// Cache coherence protocol
    #[clap(arg_enum, value_parser)]
    protocol: ProtocolKind,

    /// Path to the benchmark archive, e.g. "./blackscholes_four.zip"
    #[clap(value_parser)]
    input_file: String,

    /// Cache size in bytes
    #[clap(value_parser)]
    cache_size: usize,

    /// Cache associativity
    #[clap(value_parser)]
    associativity: usize,

    /// Cache block size in bytes
    #[clap(value_parser)]
    block_size: usize,

    /// Disable progress display
    #[clap(short, long)]
    no_progress: bool,
}

// taken from https://stackoverflow.com/a/600306
fn power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

fn check_args(args: &ProgramArgs) {
    if !power_of_two(args.cache_size) {
        panic!("Cache size must be a power of 2.");
    }
    if !power_of_two(args.associativity) {
        panic!("Associativity must be a power of 2.");
    }
    if !power_of_two(args.block_size) {
        panic!("Block size must be a power of 2.");
    }
}

fn main() {
    LOGGER.open_create("binlog");

    let args = ProgramArgs::parse();
    check_args(&args);

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

    LOGGER.write(LogEntry::InitSystem(InitSystem {
        protocol: format!("{:?}", &args.protocol),
        cache_size: args.cache_size,
        associativity: args.associativity,
        block_size: args.block_size,
        num_cores: record_streams.len(),
        archive_name: String::from(&args.input_file),
    }));
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

    LOGGER.open_read("binlog");
    LOGGER.read_to_stdout();
}
