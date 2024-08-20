use crate::{basics::{row::{Row, Value}, column::Column}};

use super::condition::chain::ConditionChain;

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

impl SelectQuery {
    pub fn get_limit(&self) -> Option<usize> {
        for extra in &self.extras {
            if let SelectExtra::Limit(n) = extra {
                return Some(*n)
            }
        }
        None
    }

    pub fn get_offset(&self) -> Option<usize> {
        for extra in &self.extras {
            if let SelectExtra::Offset(n) = extra {
                return Some(*n)
            }
        }
        None
    }

    pub fn get_order(&self) -> Option<&Order> {
        for extra in &self.extras {
            if let SelectExtra::Order(order) = extra {
                return Some(order)
            }
        }
        None
    }

    pub fn get_exclude(&self) -> Option<&Vec<String>> {
        for extra in &self.extras {
            if let SelectExtra::Exclude(cols) = extra {
                return Some(cols)
            }
        }
        None
    }

    pub fn get_where(&self) -> Option<&ConditionChain> {
        for extra in &self.extras {
            if let SelectExtra::Where(chain) = extra {
                return Some(chain)
            }
        }
        None
    }
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

impl InsertQuery {
    pub fn get_key_val(&self, key: &str) -> Option<&str> {
        self.key_vals.iter().find_map(|key_val| {
            if key_val.key == key {
                Some(key_val.val.as_str())
            } else {
                None
            }
        }) 
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.key_vals.iter().map(|key_val| key_val.key.clone()).collect()
    }
}

#[derive(Debug)]
pub struct UpdateQuery {
    pub key_vals: Vec<KeyVal>,
    pub condition_chain: ConditionChain,
}

impl UpdateQuery {
    pub fn is_valid(&self) -> bool {
        !self.key_vals.is_empty() && !self.condition_chain.is_empty()
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.key_vals.iter().map(|key_val| key_val.key.clone()).collect()
    }

    /// Transforms KeyVals (column_name, value_str) into a list of (column_index, value) tuples
    pub fn get_parsed_key_vals(&self, columns: &Vec<Column>) -> Result<Vec<(usize, Value)>, String> {
        let mut parsed = Vec::new(); 

        for key_val in &self.key_vals {
            let (column_index, column) = columns
                .iter()
                .enumerate()
                .find(|(_, c)| c.name == key_val.key)
                .ok_or(format!("Column '{}' not found", key_val.key))?;
            let value = column.validate(&key_val.val)?;
            
            parsed.push((column_index, value));
        }

        Ok(parsed)
    }
}

#[derive(Debug)]
pub struct DeleteQuery {
    pub condition_chain: ConditionChain,
    pub limit: Option<usize>,
}

impl DeleteQuery {
    pub fn is_valid(&self) -> bool {
        !self.condition_chain.is_empty()
    }
}

#[derive(Debug)]
pub enum Order {
    Ascending(String),
    Descending(String),
}

impl Order {
    pub fn get_column(&self) -> &str {
        match self {
            Self::Ascending(column) => column,
            Self::Descending(column) => column,
        }
    }

    /// Compare two rows based on the order, column should be valid and checked before calling this function
    /// - this functin will panic if the column index is out of bounds or the comparisson is invalid
    pub fn compare(&self, a: &Row, b: &Row, i: usize) -> std::cmp::Ordering {
        let a = a.get(i).expect("Row index out of bounds, invalid column index");
        let b = b.get(i).expect("Row index out of bounds, invalid column index");

        match self {
            Self::Ascending(_) => a.partial_cmp(&b).expect("Invalid comparison"),
            Self::Descending(_) => b.partial_cmp(&a).expect("Invalid comparison"),
        }
    }
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
    pub fn get_specific(&self) -> Option<&QueryType> { self.specific.as_ref() }
}
