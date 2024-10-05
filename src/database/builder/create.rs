use crate::{utils::{is_valid_name, log, disk}, database::Database, parser::Schema, basics::Table};

use super::DatabaseBuilder;

impl DatabaseBuilder {
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

}
