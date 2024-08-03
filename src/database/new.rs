use crate::database::Database;
use crate::utils::{disk, log};
use crate::parser::Schema;

impl Database {
    pub fn new_from_schema(schema: Schema) -> Self {
        log::info(format!("loading database '{}' from schema", schema.database_name));

        let mut database = Database {
            name: schema.database_name,
            tables: schema.tables,
            root_dir: schema.root_dir,
        };

        database.create_files();
        database.load_tables();

        database
    }

    fn load_tables(&mut self) {
        let path = format!("{}/tables", self.path());
        self.tables.iter_mut().for_each(|table| table.load(&path))
    }

    fn create_files(&self) {
        log::info(format!("creating database files '{}'", self.name));
        let path = self.path();

        if disk::exists(&path) {
            log::error(format!("database '{}' already exists", self.name));
            return;
        }

        disk::create_directory_all(&path);
        disk::create_directory(&format!("{}/tables", &path));

        for table in &self.tables {
            disk::create_file(&format!("{}/tables/{}.quack", &path, &table.name));
        }

        log::success(format!("database '{}' created", self.name));
    }
}
