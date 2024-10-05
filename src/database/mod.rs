mod new;
mod builder;
mod run;
mod loader;
mod creator;

// pub use builder::DatabaseBuilder;
pub use loader::DatabaseBuilder;
pub use run::{QueryResult, Run, RunOptions};

use std::{collections::HashMap, fs::File, sync::{Arc, RwLock}};

use crate::{basics::Table, function::Function};

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
    schema: File,
}

impl Database {
    /// Return new Database instance
    pub fn new(name: &str, root_dir: &str, schema: File) -> Self {
        Self {
            name: name.to_string(),
            tables: Vec::new(),
            root_dir: root_dir.to_string(),
            functions: HashMap::new(),
            schema,
        }
    }

    /// Extract the database from Arc<RwLock<Database>>,
    /// used during internal loading processes.
    ///
    /// # Safety
    /// Provided database should not have any active clones, read or write locks.
    /// It also should not be used after the extraction
    pub fn extract(database: Arc<RwLock<Self>>) -> Result<Self, String> {
        let mut db = database.write().unwrap();
        let database = Database {
            name: db.name.clone(),
            root_dir: db.root_dir.clone(),
            schema: db.schema.try_clone().or_else(|e| Err(e.to_string()))?,
            tables: db.tables.drain(..).collect(), 
            functions: db.functions.drain().collect(),
        };

        Ok(database)
    }

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
