use std::{path::PathBuf, io::{BufReader, BufWriter}, fs::File};

use crate::basics::row::Row;

pub enum LoadMode {
    Memory,
    Disk,
}

pub struct Data {
    pub rows: Vec<Row>,
    pub reader: BufReader<File>,
    pub writer: BufWriter<File>,
    pub path: PathBuf,
    pub load_mode: LoadMode,
}
