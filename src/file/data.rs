use std::{path::PathBuf, io::{BufReader, BufWriter}, fs::File};

use crate::basics::row::Row;

#[derive(Debug)]
pub enum LoadMode {
    Memory,
    Disk,
}

#[derive(Debug)]
pub struct Data {
    pub rows: Vec<Row>,
    reader: Option<BufReader<File>>,
    writer: Option<BufWriter<File>>,
    path: Option<PathBuf>,
    loaded: bool,
    pub load_mode: LoadMode,
}

impl Data {
    pub fn load(&mut self, path: PathBuf) {
        if self.loaded { return }

        let writer_file = File::options().write(true).create(true).open(&path).unwrap();
        let reader_file = File::options().read(true).open(&path).unwrap();

        let reader = BufReader::new(reader_file);
        let writer = BufWriter::new(writer_file);

        self.reader = Some(reader);
        self.writer = Some(writer);
        self.path = Some(path);

        self.loaded = true;
    }

    pub fn new(path: PathBuf) -> Data {
        let mut data = Data::default();
        data.load(path); 

        data
    } 
}

impl Default for Data {
    fn default() -> Self {
        Data {
            rows: Vec::new(),
            load_mode: LoadMode::Memory,

            reader: None,
            writer: None,
            path: None,
            loaded: false,
        }
    }
}
