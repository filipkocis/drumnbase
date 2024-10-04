use std::collections::HashMap;
use std::fs::OpenOptions;

use crate::database::Database;
use crate::parser::Schema;

impl Database {
    pub fn from_schema(name: &str, root_dir: &str, schema: Schema) -> Self {
        Database {
            name: name.to_string(),
            tables: schema.tables,
            root_dir: root_dir.to_string(),
            functions: HashMap::new(),
            // TODO: should be removed later
            schema: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(Schema::path(&format!("{root_dir}/{name}"))).unwrap()
        }
    }
}
