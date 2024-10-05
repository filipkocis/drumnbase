use std::{fs::{OpenOptions, File}, rc::Rc, sync::{RwLock, Arc}, io::Read};

use crate::{parser::Schema, utils::{log, disk}, cluster::{Cluster, ClusterSettings}, file::{purge::Purge, read::DatabaseReader}, basics::Table};

use super::{Database, Run};

mod load;
mod create;

pub struct DatabaseBuilder {
    name: String,
    root_dir: String,
}

impl DatabaseBuilder {
    pub fn new(name: &str, root_dir: &str) -> Self {
        Self {
            name: name.to_string(),
            root_dir: root_dir.to_string(),                                
        }
    }

    /// Path for the database directory as 'root_dir/name'
    pub fn path(&self) -> String {
        let separator = if self.root_dir.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", self.root_dir, separator, self.name)
    }

    /// Prepare the database for use by loading tables, purging deleted rows and adding built-in
    /// functions
    fn prepare_database(&self, database: &mut Database) -> Result<(), String> {
        let path = self.path();

        for table in &mut database.tables {
            if !disk::exists(&Table::path_for(&path, &table.name)) {
                log::error(format!("Table '{}' does ont exist on disk", table.name));
            }

            table.load(&path);
            table.read()?;
        }

        database.purge()?;
        database.add_builtin_functions();

        Ok(())
    }

    /// Load schema into database as part of loading, 
    /// shouldn't be called outside of load()
    ///
    /// Since schema loading uses Database::run, it itself will 
    /// return a Database instance with the schema loaded
    fn load_schema(&self) -> Result<Database, String> {
        let schema = self.open_schema()?;
        let mut database = Database::new(&self.name, &self.root_dir, schema);

        let mut content = String::new();
        if let Err(e) = database.schema.read_to_string(&mut content) {
            let msg = format!("Failed to read schema file for database '{}' -> {}", self.name, e);
            log::error(&msg);
            return Err(e.to_string())
        }

        let db = Arc::new(RwLock::new(database));
        let settings = &ClusterSettings::new(&self.name, &self.root_dir);
        let mut options = Cluster::root_run_options(db.clone(), settings);
        options.is_schema = true;
        if let Err(e) = Database::run(db.clone(), content, Rc::new(options)) {
            let msg = format!("Failed to load schema for database '{}' -> {}", self.name, e);
            log::error(&msg);
            return Err(msg)
        }

        let database = Database::extract(db)?; 
        Ok(database) 
    }
    
    /// Return the schema file for database at self.path()
    fn open_schema(&self) -> Result<File, String> {
        let path = Schema::path(&self.path());
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| e.to_string())
    }
}
