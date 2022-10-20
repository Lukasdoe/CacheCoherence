use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use zip::{read::ZipFile, ZipArchive};

#[derive(Debug)]
pub struct Record {
    pub label: Label,
    pub value: u32,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Label {
    Load,
    Store,
    Other,
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
    pub line_count: usize,
    _zip_archive: Box<ZipArchive<File>>,
    lines: Lines<BufReader<ZipFile<'static>>>,
}

impl Iterator for RecordStream {
    type Item = Record;
    fn next(&mut self) -> Option<Record> {
        let next = self.lines.next()?;
        if let Ok(line) = next {
            let mut parts = line.split(' ');
            let label = parts.next().unwrap();
            let value = parts.next().unwrap();
            Some(Record::new(label, value))
        } else {
            None
        }
    }
}

impl RecordStream {
    pub fn new(
        file_name: String,
        zip_archive: ZipArchive<std::fs::File>,
        count_lines: bool,
    ) -> Self {
        let mut archive = Box::new(zip_archive);

        let line_count = if count_lines {
            BufReader::new(archive.by_name(&file_name).unwrap())
                .lines()
                .count()
        } else {
            0
        };

        let zip_file = unsafe {
            std::mem::transmute::<_, ZipFile<'static>>(archive.by_name(&file_name).unwrap())
        };

        RecordStream {
            file_name,
            _zip_archive: archive,
            line_count,
            lines: BufReader::new(zip_file).lines(),
        }
    }
}
