use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Label {
    Load,
    Store,
    Other,
}

#[derive(Debug, Serialize)]
pub struct Record {
    pub label: Label,
    pub value: u32,
}

impl Record {
    fn new(label: &str, value: &str) -> Self {
        Record {
            label: Record::line_to_label(label),
            value: Record::line_to_value(value),
        }
    }

    fn line_to_label(line: &str) -> Label {
        match line {
            "0" => Label::Load,
            "1" => Label::Store,
            _ => Label::Other,
        }
    }

    fn line_to_value(line: &str) -> u32 {
        let stripped_s = line.trim_start_matches("0x");
        u32::from_str_radix(stripped_s, 16).unwrap()
    }
}
pub struct RecordStream {
    pub file_name: String,
    lines: Lines<BufReader<File>>,
}

impl Iterator for RecordStream {
    type Item = Record;
    fn next(&mut self) -> Option<Record> {
        let next = self.lines.next()?;
        if let Ok(line) = next {
            let mut parts = line.split(" ");
            let label = parts.next().unwrap();
            let value = parts.next().unwrap();
            Some(Record::new(label, value))
        } else {
            None
        }
    }
}

impl RecordStream {
    pub fn new(file_name: String, file: File) -> Self {
        RecordStream {
            file_name,
            lines: BufReader::new(file).lines(),
        }
    }
}
