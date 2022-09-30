use std::fs::File;
use std::io::{BufRead, BufReader};
use zip::ZipArchive;

#[derive(Debug)]
pub enum Label {
    Load,
    Store,
    Other,
}

#[derive(Debug)]
pub struct Record {
    pub label: Label,
    pub value: u32,
}

pub struct RecordStream {
    pub zip_archive: ZipArchive<File>,
    pub file_name: String,
}

impl RecordStream {
    pub fn start(&mut self) -> Box<dyn Iterator<Item = Record> + '_> {
        let data_file = self.zip_archive.by_name(&self.file_name).unwrap();
        let lines = BufReader::new(data_file).lines();
        return Box::new(lines.map(move |line| Record {
            label: RecordStream::line_to_label(line.as_ref().unwrap()),
            value: RecordStream::line_to_value(line.as_ref().unwrap()),
        }));
    }

    fn line_to_label(line: &str) -> Label {
        match line.split(" ").nth(0).unwrap() {
            "0" => Label::Load,
            "1" => Label::Store,
            _ => Label::Other,
        }
    }

    fn line_to_value(line: &str) -> u32 {
        let value_s = line.split(" ").nth(1).unwrap();
        let stripped_s = value_s.trim_start_matches("0x");
        u32::from_str_radix(stripped_s, 16).unwrap()
    }
}
