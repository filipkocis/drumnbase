use std::path::PathBuf;

use crate::{basics::column::Column, file::data::Data};

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub data: Data,
    pub read_only: bool,
    // pub constraints: Vec<Constraint>,
    // pub triggers: Vec<Trigger>,
    // pub indexes: Vec<Index>,
    // pub rules: Vec<Rule>,
    // pub partitions: Vec<Partition>,
    // pub comments: Vec<Comment>,
    // pub privileges: Vec<Privilege>,
}

impl Table {
    pub fn new(path: &str) -> Table {
        let path = PathBuf::from(path);

        Table {
            name: path.file_stem().unwrap().to_str().unwrap().to_owned(),
            columns: Vec::new(),
            data: Data::new(path),
            read_only: false,
        }
    }

    pub fn get_column_mut(&mut self, column_name: &str) -> Option<&mut Column> {
        self.columns.iter_mut().find(|column| column.name == column_name)
    }

    pub fn get_column(&self, column_name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == column_name)
    }
}
