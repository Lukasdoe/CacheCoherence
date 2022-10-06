use crate::log_types::*;
use bincode::config::{self, Configuration};
use bincode::serde::{decode_from_std_read, encode_into_std_write};
use std::sync::Mutex;
use std::{fs::File, fs::OpenOptions};

const DEFAULT_CONFIG: config::Configuration = config::standard();

pub struct Logger {
    file: Mutex<Option<File>>,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            file: Mutex::new(None),
        }
    }

    pub fn open_create(&self, path: &str) {
        *self.file.lock().unwrap() = Some(
            OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .append(false)
                .truncate(true)
                .open(path)
                .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
        )
    }

    pub fn open_read(&self, path: &str) {
        *self.file.lock().unwrap() = Some(
            OpenOptions::new()
                .read(true)
                .open(path)
                .expect(&format!("Could not open / create logfile \"{:?}\"", path)),
        )
    }

    pub fn write(&self, entry: LogEntry) {
        let mut file_opt = self.file.lock().unwrap();
        let file = file_opt.as_mut().unwrap();
        encode_into_std_write(&entry, file, DEFAULT_CONFIG).unwrap();
    }

    pub fn read(&self) -> Option<LogEntry> {
        let mut file_opt = self.file.lock().unwrap();
        let file = file_opt.as_mut().unwrap();
        return decode_from_std_read::<LogEntry, Configuration, File>(file, DEFAULT_CONFIG).ok();
    }

    pub fn read_to_stdout(&self) {
        let mut file_opt = self.file.lock().unwrap();
        let file = file_opt.as_mut().unwrap();
        while let Ok(next_entry) =
            decode_from_std_read::<LogEntry, Configuration, File>(file, DEFAULT_CONFIG)
        {
            println!("{:?}", next_entry);
        }
    }
}
