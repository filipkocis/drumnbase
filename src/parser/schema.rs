use crate::basics::{table::Table, column::Column};

#[derive(Debug)]
pub struct Schema {
    pub root_dir: String,
    pub database_name: String,
    pub tables: Vec<Table>,
}

impl Schema {
    // pub fn new(root_dir: String) -> Schema {
    //     Schema {
    //         root_dir,
    //         database_name: String::new(),
    //         tables: Vec::new(),
    //     }
    // }

    pub fn get_table_path(&self, table_name: &str) -> String {
        format!("{}/{}/tables/{}.quack", self.root_dir, self.database_name, table_name)
    }

    pub fn add_table(&mut self, name: &str) {
        let table = Table::new(name);
        self.tables.push(table);
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
}

impl Default for Schema {
    fn default() -> Self {
        Schema {
            root_dir: String::new(),
            database_name: String::new(),
            tables: Vec::new(),
        }
    }
}
