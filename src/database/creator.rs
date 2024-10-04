use std::io::Write;

use crate::{basics::table::Table, syntax::{ast::{Node, SDL, CreateSDL}, stringify::ToSchemaString}, utils::{disk, is_valid_name}, auth::RlsPolicy};

use super::Database;

impl Database {
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
