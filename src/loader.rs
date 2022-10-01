use std::fs::File;
use std::io::Error;

use crate::record::RecordStream;

pub struct FileLoader;

impl FileLoader {
    pub fn open(path: &str) -> Result<Vec<RecordStream>, Error> {
        let paths = std::fs::read_dir(path).unwrap();

        let mut res_vec = Vec::new();
        for path in paths {
            let path = path.unwrap();
            let file = File::open(&path.path())?;
            let file_name = String::from(path.file_name().as_os_str().to_str().unwrap());
            res_vec.push(RecordStream::new(file_name, file));
        }
        Ok(res_vec)
    }
}
