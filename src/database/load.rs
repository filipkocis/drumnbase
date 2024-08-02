use crate::database::Database;
use crate::utils::{disk, log};

impl Database {
    pub fn loadd(name: String) -> Database {
        // check fs if database exists
        let mut db = Database::default();
        // db.name = name;

        // load tables, read directory contents
        // for each file load table
        let path = format!("data/{}", name);
        let result = std::fs::read_dir(path);        

        todo!()
    }
}

impl Database {
    pub fn load(name: &str, root_dir: &str) -> Option<Database> {
        let path = format!("{}/{}", root_dir, name);
        

        if !disk::exists(&path) {
            log::error(format!("database {} does not exist", name));
            return None;
        }

        let mut db = Database {
            name: name.to_string(),
            tables: vec![], 
            root_dir: root_dir.to_string(),
        };

        disk::get_entires(&path);

        None
    }
}
