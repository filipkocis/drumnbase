use std::{path::PathBuf, io::{BufReader, BufWriter, Write}, fs::File};

use crate::{basics::row::Row, utils::log};

#[derive(Debug)]
pub enum LoadMode {
    Memory,
    Disk,
}

#[derive(Debug)]
pub struct Data {
    pub buf_rows: Vec<Row>,
    rows: Vec<Row>,
    reader: Option<BufReader<File>>,
    writer: Option<BufWriter<File>>,
    path: Option<PathBuf>,
    loaded: bool,
    pub load_mode: LoadMode,
}

impl Data {
    pub fn load(&mut self, path: PathBuf) {
        if self.loaded { return }

        log::info(format!("loading data from '{}'", path.display()));

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

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn is_empty_buf(&self) -> bool {
        self.buf_rows.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Row> {
        self.rows.iter()
    }
}

impl Data {
    pub fn write_memory(&mut self) -> Result<(), String> {
        if self.buf_rows.len() == 0 { return Ok(()) }
        if !self.loaded { return Err("data not loaded".to_string()) }

        self.buf_rows.iter().rev().for_each(|r| {
            self.rows.push(r.clone())
        });

        self.write_disk()
    }

    pub fn write_disk(&mut self) -> Result<(), String> {
        if self.buf_rows.len() == 0 { return Ok(()) }
        if !self.loaded { return Err("data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();

        self.buf_rows.reverse();
        for buf_row in self.buf_rows.pop() {
            writer.write_all(buf_row.to_string().as_bytes()).unwrap();
        }

        writer.flush().unwrap();
        Ok(())
    }
}

impl Default for Data {
    fn default() -> Self {
        Data {
            rows: Vec::new(),
            buf_rows: Vec::new(),
            load_mode: LoadMode::Memory,

            reader: None,
            writer: None,
            path: None,
            loaded: false,
        }
    }
}
