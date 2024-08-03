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
    pub reader: BufReader<File>,
    pub writer: BufWriter<File>,
    pub path: PathBuf,
    pub load_mode: LoadMode,
}

impl Data {
    pub fn new(path: PathBuf) -> Data {
        let writer_file = File::options().write(true).create(true).open(&path).unwrap();
        let reader_file = File::options().read(true).open(&path).unwrap();

        let reader = BufReader::new(reader_file);
        let writer = BufWriter::new(writer_file);

        Data {
            rows: Vec::new(),
            reader,
            writer,
            path,
            load_mode: LoadMode::Memory,
        }
    } 
}
