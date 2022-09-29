use std::io::{BufRead, Error};
use zip::ZipArchive;

use crate::types::*;

pub struct RecordStream {
    pub zip_archive: ZipArchive<std::fs::File>,
    pub file_name: String,
}

impl RecordStream {
    pub fn start(&mut self) -> Box<dyn Iterator<Item = Record> + '_> {
        let data_file = self.zip_archive.by_name(&self.file_name).unwrap();
        let lines = std::io::BufReader::new(data_file).lines();
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

pub struct FileLoader {}

impl FileLoader {
    pub fn open(path: &str) -> Result<Vec<RecordStream>, Error> {
        let zip_file = std::fs::File::open(path)?;
        let zip_archive = zip::ZipArchive::new(zip_file)?;

        let mut res_vec = Vec::new();
        for file_name in zip_archive.file_names() {
            res_vec.push(RecordStream {
                zip_archive: zip::ZipArchive::new(std::fs::File::open(path)?)?,
                file_name: String::from(file_name),
            });
        }
        Ok(res_vec)
    }
}
