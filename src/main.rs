use clap::Parser;

use cacher::{Core, FileLoader, ProtocolKind};

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

    /// cache associativity in bytes
    #[clap(value_parser)]
    associativity: usize,

    /// cache block size in bytes
    #[clap(value_parser)]
    block_size: usize,
}

fn main() {
    let args = ProgramArgs::parse();
    let mut record_streams = match FileLoader::open(&args.input_file) {
        Ok(streams) => streams,
        Err(e) => {
            println!(
                "Error during loading of the supplied input file: {:?}",
                e.to_string()
            );
            std::process::exit(e.raw_os_error().unwrap_or(1));
        }
    };

    let cores: Vec<Core> = record_streams
        .iter()
        .map(|_stream| Core::new(&args.protocol))
        .collect();

    for stream in record_streams.iter_mut() {
        println!("{:?}", stream.file_name);
        println!("{:?}", stream.start().last().unwrap());
        println!("{:?}", stream.start().count());
    }
}
