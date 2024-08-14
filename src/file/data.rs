use std::{path::PathBuf, io::{BufReader, BufWriter, Write, Read, SeekFrom, Seek}, fs::File};

use crate::{basics::{row::{Row}, column::Column}, utils::log};

#[derive(Debug, PartialEq)]
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
    /// Appends the buffer to memory rows, leaving it empty
    pub fn buffer_apply(&mut self) {
        self.rows.append(&mut self.buf_rows)
    }
}

impl Data {
    /// Seeks the writer to the end
    pub fn writer_seek_end(&mut self) -> Result<(), String> {
        if !self.loaded { return Err("Data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();
        writer.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Seeks the writer to the given position from the start
    pub fn writer_seek(&mut self, pos: u64) -> Result<(), String> {
        if !self.loaded { return Err("Data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();
        writer.seek(SeekFrom::Start(pos)).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Writes the buffer, does not seek or flush
    pub fn writer_write(&mut self, buf: &[u8]) -> Result<(), String> {
        if !self.loaded { return Err("Data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();
        writer.write_all(buf).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Flushes the writer
    pub fn writer_flush(&mut self) -> Result<(), String> {
        if !self.loaded { return Err("Data not loaded".to_string()) }

        let writer = self.writer.as_mut().unwrap();
        writer.flush().map_err(|e| e.to_string())?;

        Ok(())
    }
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

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }

    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
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
            let buf = self.buf_rows[i].convert_to_bytes(columns); 
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

        let mut i = 0;
        while let Ok(_) = reader.read_exact(buf.as_mut()) {
            let row = Row::convert_from_bytes(&buf, columns).map_err(|e| {
                let err_msg = format!("failed to convert row at {} from bytes: {}", i, e);
                log::error(&err_msg);
                e
            })?;

            self.rows.push(row);
            i += 1;
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
