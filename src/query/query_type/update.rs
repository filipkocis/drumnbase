use crate::{query::condition::chain::ConditionChain, basics::{column::Column, row::Value}};

use super::KeyVal;

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
