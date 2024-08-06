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

    pub fn get_column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    pub fn get_column_mut(&mut self, column_name: &str) -> Option<&mut Column> {
        self.columns.iter_mut().find(|column| column.name == column_name)
    }

    pub fn get_column(&self, column_name: &str) -> Option<&Column> {
        self.columns.iter().find(|column| column.name == column_name)
    }

    pub fn get_column_index(&self, column_name: &str) -> Result<usize, String> {
        self.columns
            .iter()
            .position(|column| column.name == column_name)
            .ok_or(format!("Column '{}' not found in table '{}'", column_name, self.name))
    }

    pub fn load(&mut self, database_path: &str) {
        log::info(format!("loading table '{}'", self.name));
        let table_path = Table::path_for(database_path, &self.name);
        let path_buf = PathBuf::from(table_path);
        self.data.load(path_buf);
    }
}

impl Table {
    pub fn print(&self) {
        let separator = "=".repeat(50);
        println!("{}", separator);
        self.print_info();
        println!();
        self.print_columns();
        println!();
        self.print_rows();
        println!("{}", separator);
    }

    pub fn print_info(&self) {
        println!("TABLE NAME: '{}'", self.name);
        println!("LOAD MODE: {:?}", self.data.load_mode);
        println!("READ ONLY: {}", self.read_only);
    }

    pub fn print_columns(&self) {
        for column in &self.columns {
            print!("COLUMN '{}' TYPE '{:?}' ", column.name, column.data_type);
            if !column.default.is_empty() { print!("DEFAULT='{}' ", column.default); }
            if column.not_null { print!("NOTNULL "); }
            if column.unique { print!("UNIQUE "); }
            if column.read_only { print!("READONLY "); }
            println!();
        }
    }

    pub fn print_rows(&self) {
        let spacing = 7;

        if !self.data.is_empty() {
            print!("     ");
            print!("{}", self.columns.iter().map(|c| format!("{:width$} ", c.name.to_uppercase(), width = spacing)).collect::<String>());
            println!();
        }

        for row in self.data.iter() {
            print!("ROW: ");
            for cell in row.iter() {
                print!("{:width$} ", format!("{}", cell), width = spacing);
            }
            println!();
        }
    }
}

impl Table {
    pub fn check_column_exists(&self, column_name: &str) -> Result<(), String> {
        if column_name == "*" { return Ok(()) }

        match self.get_column(column_name) {
            None => Err(format!("Column '{}' not found in table '{}'", column_name, self.name)),
            Some(_) => Ok(())
        }
    }

    pub fn check_columns_exist(&self, column_names: &Vec<String>) -> Result<(), String> {
        match column_names.iter().filter(|n| self.check_column_exists(n).is_err()).collect::<Vec<&String>>() {
            c if c.len() == 0 => Ok(()),
            invalid_columns => Err(format!("Columns {:?} not found in table '{}'", invalid_columns, self.name))
        }
    }

    pub fn get_column_indicies(&self, column_names: &Vec<String>) ->Result<Vec<usize>, String> {
        let mut indicies = Vec::new();

        for column_name in column_names {
            let index = self.get_column_index(column_name)?;
            indicies.push(index);
        }

        Ok(indicies)
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
