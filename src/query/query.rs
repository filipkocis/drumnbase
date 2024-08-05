use crate::{database::database::Database, basics::row::Row};

use super::condition::ConditionChain;


pub struct QueryResult {
    pub amount: usize,
    pub data: Vec<Row>,
}

pub enum SelectExtra {
    Where(ConditionChain),
    Order(Order),
    Limit(usize),
    Offset(usize),
    Exclude(Vec<String>),
}

impl SelectExtra {
    pub fn list() -> Vec<&'static str> {
        vec!["where", "order", "limit", "offset", "exclude"]
    }
}

pub struct SelectQuery {
    pub columns: Vec<String>,  
    pub extras: Vec<SelectExtra>,
}

pub struct InsertQuery {
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

pub struct UpdateQuery {
    pub columns: Vec<String>,
    pub values: Vec<String>,
    pub conditions: ConditionChain,
}

pub struct DeleteQuery {
    pub conditions: ConditionChain,
    pub limit: Option<usize>,
}

pub enum Order {
    Ascending(String),
    Descending(String),
}

pub enum QueryType {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}

pub struct Query {
    text: String,
    table_name: String,
    specific: Option<QueryType>,
}

impl Query {
    pub fn new(text: &str, table_name: &str) -> Self {
        Self {
            text: text.to_string(),
            table_name: table_name.to_string(),
            specific: None, 
        }
    }

    pub fn set_specific(&mut self, specific: QueryType) {
        self.specific = Some(specific);
    }

    pub fn apply_to(&self, database: &mut Database) -> Result<QueryResult, String> {
        todo!()
    }

    pub fn get_text(&self) -> &str { &self.text }
    pub fn get_table_name(&self) -> &str { &self.table_name }
}
