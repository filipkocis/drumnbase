use crate::basics::table::Table;
use crate::parser::Schema;
use crate::database::Database;
use crate::utils::{log, disk};

pub struct DatabaseBuilder {
    name: String,
    root_dir: String,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        DatabaseBuilder {
            name: String::from(""),
            root_dir: String::from("./data"),
        }
    }

    pub fn from(name: &str) -> Self {
        DatabaseBuilder::new().name(name)
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn root_dir(mut self, root_dir: &str) -> Self {
        self.root_dir = root_dir.to_string();
        self
    }
}

impl DatabaseBuilder {
    fn path(&self) -> Result<String, String> {
        if self.name.is_empty() || self.root_dir.is_empty() {
            let err_msg = format!("missing fields name='{}' root_dir='{}'", self.name, self.root_dir).to_string();
            log::error(&err_msg);
            return Err(err_msg);
        }

        Ok(format!("{}/{}", self.root_dir, self.name))
    }

    fn check_dir(&self) -> Result<(), String> {
        log::info("validating database directory path");

        let path = &self.path()?;
        if !disk::exists(path) {
            let err_msg = format!("database '{}' does not exist", self.name);
            log::error(&err_msg);
            return Err(err_msg);
        }

        Ok(())
    }

    fn check_files(&self) -> Result<(), String> {
        log::info("validating database file paths");

        let path = Schema::path(&self.path()?);
        if !disk::exists(&path) {
            let err_msg = format!("schema file for '{}' does not exist", self.name);
            log::error(&err_msg);
            return Err(err_msg);
        }

        Ok(())
    }

    fn check_schema_files(&self, schema: &Schema) -> Result<(), String> {
        log::info("validating schema table file paths");

        let path = &self.path()?;
        let paths: Vec<String> = schema.tables
            .iter()
            .map(|table| Table::path_for(path, &table.name))
            .collect();

        let mut missing_files = false;
        paths.iter().for_each(|path| {
            if !disk::exists(path) {
                missing_files = true;
                log::error(format!("missing file '{}'", path));
            }
        });

        if missing_files { 
            Ok(()) 
        } else { 
            let err_msg = format!("file check failed, schema doesn't match");
            log::error(&err_msg);
            Err(err_msg)
        }
    }

    pub fn load(&self) -> Result<Database, String> {
        log::info(format!("loading database '{}' schema", self.name));

        self.check_dir()?;
        self.check_files()?;

        let database_path = self.path()?;
        let schema_path = Schema::path(&database_path);
        let mut schema = Schema::load_from_file(&schema_path)?;   
         
        self.check_schema_files(&schema)?;

        let tables_path = Table::path(&database_path);
        schema.tables.iter_mut().for_each(|table| table.load(&tables_path));

        let database = Database::from_schema(&self.name, &self.root_dir, schema);

        Ok(database)
    }
}
