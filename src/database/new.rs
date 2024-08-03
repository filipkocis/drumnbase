use crate::database::Database;
use crate::parser::Schema;

impl Database {
    pub fn from_schema(name: &str, root_dir: &str, schema: Schema) -> Self {
        Database {
            name: name.to_string(),
            tables: schema.tables,
            root_dir: root_dir.to_string(),
        }
    }
}
