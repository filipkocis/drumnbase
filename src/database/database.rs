use crate::basics::table::Table;

#[derive(Debug)]
pub struct Database {
    pub name: String,    
    pub tables: Vec<Table>,
    pub root_dir: String,
    // pub views: Vec<View>,
    // pub functions: Vec<Function>,
    // pub procedures: Vec<Procedure>,
    // pub triggers: Vec<Trigger>,
    // pub indexes: Vec<Index>,
    // pub sequences: Vec<Sequence>,
    // pub roles: Vec<Role>,
    // pub users: Vec<User>,
    // pub groups: Vec<Group>,
    // pub privileges: Vec<Privilege>,
    // pub constraints: Vec<Constraint>,
    // pub schemas: Vec<Schema>,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            name: String::from(""),
            tables: vec![],
            root_dir: String::from("data"),
        }
    } 
}

impl Database {
    pub fn path(&self) -> String {
        format!("{}/{}", self.root_dir, self.name) 
    }
}
