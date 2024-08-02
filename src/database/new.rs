use crate::basics::table::Table;
use crate::database::Database;
use crate::utils::{disk, log};
use crate::parser::Schema;

impl Database {
    pub fn new_from_schema(name: String, schema: Schema) -> Self {
        let database = Database {
            name,
            tables: schema.tables,
            root_dir: schema.root_dir,
        };

        database.create_files();
        database
    }

    fn create_files(&self) {
        let path = self.path();

        if disk::exists(&path) {
            log::error(format!("database {} already exists", self.name));
            return;
        }

        disk::create_directory_all(&path);
        disk::create_directory(&format!("{}/tables", &path));

        for table in &self.tables {
            disk::create_directory(&format!("{}/tables/{}.quack", &path, &table.name));
        }
    }
}
