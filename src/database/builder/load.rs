use crate::{utils::{log, disk}, database::Database, basics::Table, parser::Schema};

use super::DatabaseBuilder;

impl DatabaseBuilder {
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
