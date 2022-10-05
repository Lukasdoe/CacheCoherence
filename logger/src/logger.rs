use bincode::config::{self, Configuration};
use bincode::error::DecodeError;
use bincode::serde::{decode_from_std_read, encode_into_std_write};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use std::{fs::File, fs::OpenOptions};

pub struct Logger {
    file: File,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum EntryType {
    EnvInfo,
    Step,
    CoreInit,
    CoreState,
    CacheState,
    CacheAccess,
    CacheUpdate,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CoreInit {
    file_name: String,
    id: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnvInfo {
    protocol: String,
    cache_size: usize,
    associativity: usize,
    block_size: usize,
    num_cores: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Step {
    clk: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CoreState {
    id: usize,
    record: Option<String>,
    alu_cnt: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CacheState {
    core_id: usize,
    cnt: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CacheAccess {
    core_id: usize,

    /// hit = true, miss = false
    hit_or_miss: bool,
    tag: u32,
    index: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CacheUpdate {
    core_id: usize,

    old_tag: Option<u32>,
    new_tag: u32,
    index: usize,
    block: usize,
}

const DEFAULT_CONFIG: config::Configuration = config::standard();

impl Logger {
    pub fn create(path: &str) -> Logger {
        Logger {
            file: OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .append(false)
                .truncate(true)
                .open(path)
                .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
        }
    }

    pub fn open_read(path: &str) -> Logger {
        Logger {
            file: OpenOptions::new()
                .read(true)
                .open(path)
                .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
        }
    }

    fn write<E: Serialize>(&mut self, entry_type: EntryType, entry: E) {
        encode_into_std_write(&entry_type, &mut self.file, DEFAULT_CONFIG).unwrap();
        encode_into_std_write(&entry, &mut self.file, DEFAULT_CONFIG).unwrap();
    }

    pub fn read_next_type(&mut self) -> Result<EntryType, DecodeError> {
        return decode_from_std_read::<EntryType, Configuration, File>(
            &mut self.file,
            DEFAULT_CONFIG,
        );
    }

    pub fn read_next<D: DeserializeOwned>(&mut self) -> Result<D, DecodeError> {
        return decode_from_std_read::<D, Configuration, File>(&mut self.file, DEFAULT_CONFIG);
    }

    pub fn read_to_stdout(&mut self) {
        loop {
            let next_entry_info = decode_from_std_read::<EntryType, Configuration, File>(
                &mut self.file,
                DEFAULT_CONFIG,
            );

            if let Err(_) = next_entry_info {
                break;
            }

            match next_entry_info.unwrap() {
                EntryType::EnvInfo => println!(
                    "{:?}",
                    decode_from_std_read::<EnvInfo, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::Step => println!(
                    "{:?}",
                    decode_from_std_read::<Step, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CoreInit => println!(
                    "{:?}",
                    decode_from_std_read::<CoreInit, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CoreState => println!(
                    "{:?}",
                    decode_from_std_read::<CoreState, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheState => println!(
                    "{:?}",
                    decode_from_std_read::<CacheState, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheAccess => println!(
                    "{:?}",
                    decode_from_std_read::<CacheAccess, Configuration, File>(
                        &mut self.file,
                        DEFAULT_CONFIG
                    )
                    .unwrap()
                ),
                EntryType::CacheUpdate => println!(
                    "{:?}",
                    decode_from_std_read::<CacheUpdate, Configuration, File>(
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

    pub fn log_cache_state(&mut self, core_id: usize, cnt: u32) {
        self.write(EntryType::CacheState, CacheState { core_id, cnt })
    }

    pub fn log_cache_access(&mut self, core_id: usize, hit_or_miss: bool, tag: u32, index: usize) {
        self.write(
            EntryType::CacheAccess,
            CacheAccess {
                core_id,
                hit_or_miss,
                tag,
                index,
            },
        )
    }

    pub fn log_cache_update(
        &mut self,
        core_id: usize,
        old_tag: Option<u32>,
        new_tag: u32,
        index: usize,
        block: usize,
    ) {
        self.write(
            EntryType::CacheUpdate,
            CacheUpdate {
                core_id,
                old_tag,
                new_tag,
                index,
                block,
            },
        )
    }
}
