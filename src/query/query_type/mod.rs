pub mod select;
pub mod insert;
pub mod update;
pub mod delete;

use crate::basics::row::Row;

pub use self::{
    select::*, 
    insert::*, 
    update::*, 
    delete::*,
};

/// KeyVal is a key-value pair used in InsertQuery and UpdateQuery
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

/// Order is used to specify the order in a SelectQuery
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
