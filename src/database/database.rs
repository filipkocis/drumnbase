use std::collections::HashMap;

use crate::{basics::table::Table, function::Function};

#[derive(Debug)]
pub struct Database {
    pub name: String,    
    pub tables: Vec<Table>,
    pub root_dir: String,
    // pub views: Vec<View>,
    pub functions: HashMap<String, Function>,
    // pub procedures: Vec<Procedure>,
    // pub triggers: Vec<Trigger>,
    // pub indexes: Vec<Index>,
    // pub sequences: Vec<Sequence>,
    // pub roles: Vec<Role>,
    // pub users: Vec<User>,
    // pub groups: Vec<Group>,
    // pub privileges: Vec<Privilege>,
    // pub constraints: Vec<Constraint>,
    // pub schemas: Vec<Schema>,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            name: String::from(""),
            tables: vec![],
            root_dir: String::from("data"),
            functions: HashMap::new(),
        }
    } 
}

impl Database {
    pub fn path(&self) -> String {
        format!("{}/{}", self.root_dir, self.name) 
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|table| table.name == name)
    }
}
