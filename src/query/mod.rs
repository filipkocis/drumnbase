mod query_type;
pub mod condition;
pub mod parser;
pub mod runner;

pub use query_type::*;

use crate::basics::row::Row;

/// Query 
#[derive(Debug)]
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

    pub fn get_text(&self) -> &str { &self.text }
    pub fn get_table_name(&self) -> &str { &self.table_name }
    pub fn get_specific(&self) -> Option<&QueryType> { self.specific.as_ref() }
}

/// QueryType is the specific type of a Query 
#[derive(Debug)]
pub enum QueryType {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}

/// QueryResult is returned from executing a Query
pub struct QueryResult {
    pub amount: usize,
    pub data: Vec<Row>,
}

impl QueryResult {
    pub fn from(data: Vec<Row>) -> Self {
        Self {
            amount: data.len(),
            data
        }
    }

    pub fn with_amount(amount: usize) -> Self {
        Self {
            amount,
            data: vec![],
        }
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self {
            amount: 0,
            data: vec![],
        }
    }
}
