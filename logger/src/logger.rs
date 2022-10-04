use bincode::{
    config::{self, Configuration},
    error::DecodeError,
    Decode, Encode,
};
use std::{fs::File, fs::OpenOptions, io::Seek};

pub struct Logger {
    file: File,
}

#[derive(Decode, Encode, Debug)]
pub enum EntryType {
    EnvInfo,
    Step,
    CoreInit,
    CoreState,
    CacheState,
    CacheAccess,
    CacheUpdate,
}

#[derive(Decode, Encode, Debug)]
pub struct CoreInit {
    file_name: String,
    id: usize,
}

#[derive(Decode, Encode, Debug)]
pub struct EnvInfo {
    protocol: String,
    cache_size: usize,
    associativity: usize,
    block_size: usize,
    num_cores: usize,
}

#[derive(Decode, Encode, Debug)]
pub struct Step {
    clk: u32,
}

#[derive(Decode, Encode, Debug)]
pub struct CoreState {
    id: usize,
    record: Option<String>,
    alu_cnt: u32,
}

#[derive(Decode, Encode, Debug)]
pub struct CacheState {
    cnt: u32,
}

#[derive(Decode, Encode, Debug)]
pub struct CacheAccess {
    /// hit = true, miss = false
    hit_or_miss: bool,
    tag: u32,
    index: usize,
}

#[derive(Decode, Encode, Debug)]
pub struct CacheUpdate {
    old_tag: Option<u32>,
    new_tag: u32,
    index: usize,
    block: usize,
}

const DEFAULT_CONFIG: config::Configuration = config::standard();

impl Logger {
    pub fn new(path: &str) -> Logger {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .append(false)
            .truncate(true)
            .open(path)
            .expect(&format!("Could not open / create logfile \"{:?}\"", path));
        Logger { file: file }
    }

    fn write<E: Encode>(&mut self, entry_type: EntryType, entry: E) {
        bincode::encode_into_std_write(&entry_type, &mut self.file, DEFAULT_CONFIG).unwrap();
        bincode::encode_into_std_write(&entry, &mut self.file, DEFAULT_CONFIG).unwrap();
    }

    /// Does not rewind log file.
    pub fn read_next_type(&mut self) -> Result<EntryType, DecodeError> {
        return bincode::decode_from_std_read::<EntryType, Configuration, File>(
            &mut self.file,
            DEFAULT_CONFIG,
        );
    }

    /// Does not rewind log file.
    pub fn read_next<D: Decode>(&mut self) -> Result<D, DecodeError> {
        return bincode::decode_from_std_read::<D, Configuration, File>(
            &mut self.file,
            DEFAULT_CONFIG,
        );
    }

    /// Rewinds log file.
    pub fn read_to_stdout(&mut self) {
        self.file.rewind().unwrap();
        loop {
            let next_entry_info = bincode::decode_from_std_read::<EntryType, Configuration, File>(
                &mut self.file,
                DEFAULT_CONFIG,
            );

            if let Err(_) = next_entry_info {
                break;
            }

            match next_entry_info.unwrap() {
                EntryType::EnvInfo => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<EnvInfo, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::Step => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<Step, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CoreInit => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<CoreInit, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CoreState => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<CoreState, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheState => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<CacheState, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheAccess => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<CacheAccess, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheUpdate => println!(
                    "{:?}",
                    bincode::decode_from_std_read::<CacheUpdate, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
            }
        }
    }

    /* Logging functions */
    pub fn log_env(
        &mut self,
        protocol: String,
        cache_size: usize,
        associativity: usize,
        block_size: usize,
        num_cores: usize,
    ) {
        self.write(
            EntryType::EnvInfo,
            EnvInfo {
                protocol,
                cache_size,
                associativity,
                block_size,
                num_cores,
            },
        )
    }

    pub fn log_step(&mut self, clk: u32) {
        self.write(EntryType::Step, Step { clk });
    }

    pub fn log_core_init(&mut self, file_name: &str, id: usize) {
        self.write(
            EntryType::CoreInit,
            CoreInit {
                file_name: String::from(file_name),
                id,
            },
        )
    }

    pub fn log_core_state(&mut self, id: usize, record: Option<String>, alu_cnt: u32) {
        self.write(
            EntryType::CoreState,
            CoreState {
                id,
                record,
                alu_cnt,
            },
        )
    }

    pub fn log_cache_state(&mut self, cnt: u32) {
        self.write(EntryType::CacheState, CacheState { cnt })
    }

    pub fn log_cache_access(&mut self, hit_or_miss: bool, tag: u32, index: usize) {
        self.write(
            EntryType::CacheAccess,
            CacheAccess {
                hit_or_miss,
                tag,
                index,
            },
        )
    }

    pub fn log_cache_update(
        &mut self,
        old_tag: Option<u32>,
        new_tag: u32,
        index: usize,
        block: usize,
    ) {
        self.write(
            EntryType::CacheUpdate,
            CacheUpdate {
                old_tag,
                new_tag,
                index,
                block,
            },
        )
    }
}
