use crate::log_types::*;
use bincode::config::{self};
use bincode::serde::{decode_from_reader, encode_into_std_write};
use std::io::{BufReader, BufWriter};
use std::sync::Mutex;
use std::{fs::File, fs::OpenOptions};

const DEFAULT_CONFIG: config::Configuration = config::standard();

/// Write-only logger
pub struct WLogger {
    file: Mutex<BufWriter<File>>,
}

/// Read-only logger
pub struct RLogger {
    file: BufReader<File>,
}

impl WLogger {
    pub fn new(path: &str) -> WLogger {
        WLogger {
            file: Mutex::new(BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create(true)
                    .append(false)
                    .truncate(true)
                    .open(path)
                    .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
            )),
        }
    }

    pub fn write(&self, entry: LogEntry) {
        let mut file_opt = self.file.lock().unwrap();
        encode_into_std_write(&entry, &mut *file_opt, DEFAULT_CONFIG).unwrap();
    }
}

impl RLogger {
    pub fn new(path: &str) -> RLogger {
        RLogger {
            file: BufReader::new(
                OpenOptions::new()
                    .read(true)
                    .open(path)
                    .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
            ),
        }
    }

    pub fn read(&mut self) -> Option<LogEntry> {
        return decode_from_reader(&mut self.file, DEFAULT_CONFIG).ok();
    }

    pub fn read_to_stdout(&mut self) {
        while let Some(next_entry) = self.read() {
            println!("{:?}", next_entry);
        }
    }
}
