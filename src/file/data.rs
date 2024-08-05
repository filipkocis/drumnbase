use std::{path::PathBuf, io::{BufReader, BufWriter, Write, Read}, fs::File};

use crate::{basics::{row::{Row}, column::Column}, utils::log};

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

        let writer_file = File::options().append(true).create(true).open(&path).unwrap();
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
    pub fn write_memory(&mut self, columns: &Vec<Column>) -> Result<(), String> {
        if self.buf_rows.len() == 0 { return Ok(()) }
        if !self.loaded { return Err("data not loaded".to_string()) }

        self.buf_rows.iter().for_each(|r| {
            self.rows.push(r.clone())
        });

        self.write_disk(columns)
    }

    pub fn write_disk(&mut self, columns: &Vec<Column>) -> Result<(), String> {
        if self.buf_rows.len() == 0 { return Ok(()) }
        if !self.loaded { return Err("data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();

        for i in 0..self.buf_rows.len() {
            let mut buf = self.buf_rows[i].convert_to_bytes(columns); 
            buf.push(b'\n');

            writer.write_all(&buf).unwrap();
        }
        self.buf_rows.clear();

        writer.flush().unwrap();
        Ok(())
    }
}

impl Data {
    pub fn read_memory(&mut self, columns: &Vec<Column>) -> Result<(), String> {
        if !self.loaded { return Err("data not loaded".to_string()) }

        let reader = self.reader.as_mut().unwrap();

        let entry_size = columns.iter().fold(0, |acc, c| acc + c.length);
        let mut buf = vec![0u8; entry_size as usize];

        while let Ok(_) = reader.read_exact(buf.as_mut()) {
            let row = Row::convert_from_bytes(&buf, columns)?;
            self.rows.push(row);
        }

        Ok(())
    }

    /// Read data in load_mode 'Disk'
    /// - this functions does nothing, to store data in memory use load_mode 'Memory' which uses 'read_memory' function
    pub fn read_disk(&mut self, _: &Vec<Column>) -> Result<(), String> {
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
