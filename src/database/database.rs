use std::{collections::HashMap, fs::File, io::Write};

use crate::{basics::table::Table, function::Function, syntax::{ast::{Node, SDL, CreateSDL}, stringify::ToSchemaString}, utils::{disk, is_valid_name}, auth::RlsPolicy};

#[derive(Debug)]
pub struct Database {
    pub name: String,    
    pub tables: Vec<Table>,
    pub root_dir: String,
    // pub views: Vec<View>,
    pub functions: HashMap<String, Function>,
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
    pub(super) schema: File,
}

// impl Default for Database {
//     fn default() -> Self {
//         Database {
//             name: String::from(""),
//             tables: vec![],
//             root_dir: String::from("data"),
//             functions: HashMap::new(),
//
//             schema: File::new
//         }
//     } 
// }

impl Database {
    pub fn path(&self) -> String {
        format!("{}/{}", self.root_dir, self.name) 
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|table| table.name == name)
    }

    /// Create a new physical table in the database, and update the schema file
    pub fn create_table(&mut self, table: Table) -> Result<(), String> {
        if self.tables.iter().any(|t| t.name == table.name) {
            return Err(format!("Table {} already exists", table.name))
        }

        if !is_valid_name(&table.name) {
            return Err("Table name invalid".to_string())
        }

        let node = Node::SDL(SDL::Create(CreateSDL::Table { name: table.name.clone(), columns: table.columns.clone() }));
        let mut schema = node.to_schema_string(0)?;
        schema.push_str("\n");

        let table_path = Table::path_for(&self.path(), &table.name);
        if let Err(e) = disk::create_file(&table_path) {
            return Err(format!("Error creating table file: {}", e))
        }

        if let Err(e) = self.schema.write_all(schema.as_bytes()) {
            return Err(format!("Error writing schema: {}", e))
        }

        self.tables.push(table);

        Ok(())
    }

    /// Create a new rls policy in the database, and update the schema file
    pub fn create_rls_policy(&mut self, table_name: &str, policy: RlsPolicy) -> Result<(), String> {
        let table = match self.get_table(table_name) {
            Some(table) => table,
            None => return Err(format!("Table {} does not exist", table_name))
        };

        if table.policies.contains_key(&policy.name) {
            return Err(format!("Policy {} already exists", policy.name))
        }

        if !is_valid_name(&policy.name) {
            return Err("Policy name invalid".to_string())
        }

        let node = Node::SDL(SDL::Create(CreateSDL::RlsPolicy { 
            table: table_name.to_string(), 
            policy: policy.clone().into()
        })); 

        let mut schema = node.to_schema_string(0)?;
        schema.push('\n');

        if let Err(e) = self.schema.write_all(schema.as_bytes()) {
            return Err(format!("Error writing schema: {}", e))
        }

        let table = self.get_table_mut(table_name).expect("Table should exist");
        table.policies.insert(policy.name.clone(), policy);

        Ok(())
    }
}
