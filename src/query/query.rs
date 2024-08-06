use crate::{basics::row::Row};

use super::condition::ConditionChain;


pub struct QueryResult {
    pub amount: usize,
    pub data: Vec<Row>,
}

#[derive(Debug)]
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

    pub fn unwrap_chain(self) -> Result<ConditionChain, String> {
        match self {
            Self::Where(chain) => Ok(chain),
            _ => Err(format!("Expected Where, got {:?}", self))
        }
    }

    pub fn unwrap_limit(self) -> Result<usize, String> {
        match self {
            Self::Limit(n) => Ok(n),
            _ => Err(format!("Expected Limit, got {:?}", self))
        }
    }
}

#[derive(Debug)]
pub struct SelectQuery {
    pub columns: Vec<String>,  
    pub extras: Vec<SelectExtra>,
}

#[derive(Debug)]
pub struct KeyVal {
    pub key: String,
    pub val: String,
}

impl KeyVal {
    pub fn from(key: String, val: String) -> Self {
        Self { key, val }
    }
}

#[derive(Debug)]
pub struct InsertQuery {
    pub key_vals: Vec<KeyVal>,
}

#[derive(Debug)]
pub struct UpdateQuery {
    pub key_vals: Vec<KeyVal>,
    pub conditions: ConditionChain,
}

#[derive(Debug)]
pub struct DeleteQuery {
    pub conditions: ConditionChain,
    pub limit: Option<usize>,
}

#[derive(Debug)]
pub enum Order {
    Ascending(String),
    Descending(String),
}

#[derive(Debug)]
pub enum QueryType {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}

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
}
