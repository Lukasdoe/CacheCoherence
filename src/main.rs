use cacher::{FileLoader, ProtocolKind, System};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version,
    about = "\x1b[1mCACHER\x1b[0m - \x1b[1mCA\x1b[0mche \x1b[1mC\x1b[0mo\x1b[1mH\x1b[0merence \x1b[1mE\x1b[0mmulato\x1b[1mR\x1b[0m",
    long_about = None)]
struct ProgramArgs {
    /// cache coherence protocol
    #[clap(arg_enum, value_parser)]
    protocol: ProtocolKind,

    /// path to the benchmark archive, e.g. "./blackscholes_four.zip"
    #[clap(value_parser)]
    input_file: String,

    /// cache size in bytes
    #[clap(value_parser)]
    cache_size: usize,

    /// cache associativity
    #[clap(value_parser)]
    associativity: usize,

    /// cache block size in bytes
    #[clap(value_parser)]
    block_size: usize,
}

fn power_of_two(x: usize) -> bool {
    return (x != 0) && ((x & (x - 1)) == 0);
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
    let args = ProgramArgs::parse();
    check_args(&args);

    let record_streams = match FileLoader::open(&args.input_file) {
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
    );

    loop {
        if system.update() {
            break;
        }
    }

    logger::Logger::open_read("binlog").read_to_stdout();
}
