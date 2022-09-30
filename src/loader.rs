use std::fs::File;
use std::io::Error;
use zip::ZipArchive;

use crate::record::RecordStream;

pub struct FileLoader;

impl FileLoader {
    pub fn open(path: &str) -> Result<Vec<RecordStream>, Error> {
        let zip_file = File::open(path)?;
        let zip_archive = ZipArchive::new(zip_file)?;

        let mut res_vec = Vec::new();
        for file_name in zip_archive.file_names() {
            let zip_archive = ZipArchive::new(File::open(path)?)?;
            let file_name = String::from(file_name);
            res_vec.push(RecordStream::new(file_name, zip_archive));
        }
        Ok(res_vec)
    }
}
