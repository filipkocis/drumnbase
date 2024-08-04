use crate::basics::{table::Table, column::Column};

use super::{SimpleParser, Parser};

#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn path(database_path: &str) -> String {
        format!("{}/schema.bob", database_path)
    }

    pub fn add_table(&mut self, name: &str) -> Result<(), String> {
        if self.tables.iter().any(|table| table.name == name) {
            return Err(format!("Table already exists: {}", name))
        }

        let table = Table::new(name);
        self.tables.push(table);

        Ok(())
    }

    pub fn delete_table(&mut self, name: &str) {
        self.tables.retain(|table| table.name != name);

        // remove file because adding a table create the file (bufreader and bufwriter)
        // let path = format!("{}/tables/{}.quack", self.root_dir, name);
        // let _ = std::fs::remove_file(path);
    }

    pub fn get_table(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|table| table.name == name)
    }

    pub fn get_column(&mut self, table_name: &str, column_name: &str) -> Option<&mut Column> {
        self.get_table(table_name)
            .and_then(|table| table.columns
                .iter_mut()
                .find(|column| column.name == column_name)
            )
    }

    pub fn load_from_file(path: &str) -> Result<Schema, String> {
        SimpleParser::parse_file(path)
    }

    pub fn load_from_text(text: &str) -> Result<Schema, String> {
        SimpleParser::parse(text)
    }
}

impl Default for Schema {
    fn default() -> Self {
        Schema {
            tables: Vec::new(),
        }
    }
}
