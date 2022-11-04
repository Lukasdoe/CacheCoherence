use cacher::{Analyzer, FileLoader, ProtocolKind, System};
use clap::Parser;

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
    #[clap(value_parser, default_value_t = 4096)]
    cache_size: usize,

    /// Cache associativity
    #[clap(value_parser, default_value_t = 2)]
    associativity: usize,

    /// Cache block size in bytes
    #[clap(value_parser, default_value_t = 32)]
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
    if (args.cache_size / args.associativity) < args.block_size {
        panic!("Each cache set should be big enough to at least hold one block. (CacheSize / Associativity) < BlockSize");
    }
    if (args.cache_size / args.associativity) % args.block_size != 0 {
        panic!("Cache set size has to be multiple of the block size. (CacheSize / Associativity) mod BlockSize != 0");
    }
}

fn main() {
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
    system.hide_progress();

    let mut analyzer = Analyzer::new();
    analyzer.digest(system);
    println!(
        "\n#################\n\
         Analysis Results:\n\
         #################\n"
    );
    println!("{}", analyzer.pretty_print());
}
