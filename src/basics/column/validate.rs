use super::{Column, ColumnType};

use crate::basics::Value;

impl Column {
    /// Validates and parses a string value into Value 
    pub fn validate(&self, value: &str) -> Result<Value, String> {
        self.data_type.parse(value)
    }

    /// Validates and parses a string value into Value, 
    /// if string value is None, it returns Value::Null if possible
    pub fn validate_option(&self, value: &Option<String>) -> Result<Value, String> {
        if let Some(value) = value {
            return self.validate(&value)
        } else {
            if self.not_null {
                return Err(format!("Column '{}' does not allow NULL values", self.name))
            }

            Ok(Value::Null)
        }
    }
}

impl Column {
    /// Validates a Value against the column constraints and data_type
    pub fn validate_value(&self, value: &Value) -> Result<(), String> {
        todo!()
        // match value {
        //     Value::Array(array) => {
        //         if let ColumnType::Array(t) = &self.da
        //     }
        // }
    }
}
