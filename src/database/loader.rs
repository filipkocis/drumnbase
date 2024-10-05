use std::{fs::{OpenOptions, File}, rc::Rc, io::Read, sync::{Arc, RwLock}};

use crate::{utils::{log, is_valid_name, disk}, basics::Table, parser::Schema, database::Run, cluster::{Cluster, ClusterSettings}, file::{read::DatabaseReader, purge::Purge}};

use super::Database;

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

    /// Create a new database at self.path()
    pub fn create(&self) -> Result<Database, String> {
        if !is_valid_name(&self.name) {
            return Err(format!("Invalid database name '{}'", self.name))
        }

        log::info(format!("Creating database '{}' in '{}'", self.name, self.root_dir));

        let database_path = self.path();
        let schema_path = Schema::path(&database_path);
        let tables_path = Table::path(&database_path);

        if disk::exists(&database_path) {
            let msg = format!("Database '{}' already exists", self.name);
            log::error(&msg);
            return Err(msg);
        }

        let clean_up = |e| {
            log::error(format!("Failed to create database '{}' -> {}", self.name, e));
            disk::remove_directory_all(&database_path)?;
            Err(e)
        };

        // Create database directory
        disk::create_directory_all(&database_path).or_else(clean_up)?;
        // Create tables directory
        disk::create_directory(&tables_path).or_else(clean_up)?;
        // Create schema file
        disk::create_file(&schema_path).or_else(clean_up)?;

        let schema = match self.open_schema() {
            Ok(schema) => schema,
            Err(e) => { let _ = clean_up(e.clone()); return Err(e) }
        };

        let mut database = Database::new(&self.name, &self.root_dir, schema);
        self.prepare_database(&mut database)?;

        log::success(format!("Created database '{}'", self.name));
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

    /// Prepare the database for use by loading tables, purging deleted rows and adding built-in
    /// functions
    pub fn prepare_database(&self, database: &mut Database) -> Result<(), String> {
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

    /// Load an existing database at self.path()
    pub fn load(&self) -> Result<Database, String> {
        log::info(format!("Loading database '{}' at '{}'", self.name, self.root_dir)); 

        self.check_dir()?;
        self.check_dirs()?;
        self.check_files()?;

        let mut database = self.load_schema()?;
        self.prepare_database(&mut database)?;

        log::success(format!("Loaded database '{}'", self.name));
        Ok(database)
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

    /// Check if database directory exists
    fn check_dir(&self) -> Result<(), String> {
        log::info("Validating database directory path");

        let path = self.path();
        if !disk::exists(&path) {
            let msg = format!("Database '{}' does not exist", self.name);
            log::error(&msg);
            return Err(msg);
        }

        Ok(())
    }

    /// Check if necessary database directories exist (e.g. tables directory)
    fn check_dirs(&self) -> Result<(), String> {
        log::info("Validating database directory paths");

        let tables_path = Table::path(&self.path());
        if !disk::exists(&tables_path) {
            let msg = format!("Tables directory for database '{}' does not exist", self.name);
            log::error(&msg);
            return Err(msg);
        }

        Ok(())
    }

    /// Check if necessary database files exist (e.g. schema file)
    fn check_files(&self) -> Result<(), String> {
        log::info("Validating database file paths");

        let path = Schema::path(&self.path());
        if !disk::exists(&path) {
            let msg = format!("Schema file for database '{}' does not exist", self.name);
            log::error(&msg);
            return Err(msg);
        }

        Ok(())
    }
}
