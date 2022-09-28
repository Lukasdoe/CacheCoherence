extern crate clap;
use clap::{arg_enum, Parser, ValueEnum};

arg_enum! {
    #[derive(Clone, Debug, ValueEnum)]
    pub enum Protocol {
        MESI,
        Dragon,
    }
}

#[derive(Parser, Debug)]
#[clap(version, 
    about = "\x1b[1mCACHER\x1b[0m - \x1b[1mCA\x1b[0mche \x1b[1mC\x1b[0mo\x1b[1mH\x1b[0merence \x1b[1mE\x1b[0mmulato\x1b[1mR\x1b[0m", 
    long_about = None)]
struct ProgramArgs {
    /// cache coherence protocol
    #[clap(arg_enum, value_parser)]
    protocol: Protocol,

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
    println!("Supplied args: {:?}", args);
}
