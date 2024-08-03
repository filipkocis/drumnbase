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

    /// returns the path to the tables directory
    pub fn path(database_path: &str) -> String {
        format!("{}/tables", database_path)
    }

    /// returns the path to the table file
    pub fn path_for(database_path: &str, table_name: &str) -> String {
        format!("{}/{}.quack", Table::path(database_path), table_name)
    }

    pub fn get_column_mut(&mut self, column_name: &str) -> Option<&mut Column> {
        self.columns.iter_mut().find(|column| column.name == column_name)
    }

    pub fn get_column(&self, column_name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == column_name)
    }

    pub fn load(&mut self, database_path: &str) {
        log::info(format!("loading table '{}'", self.name));
        let table_path = Table::path_for(database_path, &self.name);
        let path_buf = PathBuf::from(table_path);
        self.data.load(path_buf);
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
