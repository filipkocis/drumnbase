use std::path::PathBuf;

use crate::{basics::column::Column, file::data::Data, utils::log};

#[derive(Debug)]
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
    pub fn new(name: &str) -> Table {
        let mut table = Table::default();
        table.name = name.to_string();
        
        table
    }

    pub fn get_column_mut(&mut self, column_name: &str) -> Option<&mut Column> {
        self.columns.iter_mut().find(|column| column.name == column_name)
    }

    pub fn get_column(&self, column_name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == column_name)
    }

    pub fn load(&mut self, path: &str) {
        log::info(format!("loading table '{}'", self.name));
        let path = format!("{}/{}.quack", path, self.name);
        let path = PathBuf::from(path);
        self.data.load(path);
    }
}

impl Default for Table {
    fn default() -> Self {
        Table {
            name: String::new(),
            columns: Vec::new(),
            data: Data::default(),
            read_only: false,
        }
    }
}
