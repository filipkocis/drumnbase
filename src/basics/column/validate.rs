use super::{Column, ColumnType, NumericType, TimestampType, TextType};

use crate::basics::{Value, value::{NumericValue, TimestampValue}};

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

pub trait Validate {
    /// Validates a Value against the column constraints and data_type
    /// It checks if the value equals the column's data type
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String>;
}

impl Column {
    /// Validates a Value against the column's data type
    /// It checks if the value equals the column's data type
    pub fn validate_value(&self, value: &Value) -> Result<(), String> {
        value.validate(self, &self.data_type)
    }
}

impl Validate for Value {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        match self {
            Value::Text(v) => v.validate(c, ct),
            Value::Numeric(v) => v.validate(c, ct),
            Value::Timestamp(v) => v.validate(c, ct),
            Value::Boolean(v) => v.validate(c, ct),
            Value::Binary(v) => v.validate(c, ct),
            Value::Array(v) => v.validate(c, ct),
            Value::Enum(v) => v.validate(c, ct),
            Value::UUID(v) => v.validate(c, ct),
            Value::Null => {
                if c.not_null {
                    return Err(format!("Column '{}' does not allow NULL values", c.name))
                }

                Ok(())
            }
        }
    }
}

impl Validate for str {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        let data_type = match ct {
            ColumnType::Text(t) => t,
            _ => return Err(format!("Column '{}' is not a text type", c.name))
        };

        match data_type {
            TextType::Char => {
                if self.len() != 1 {
                    return Err(format!("Invalid char length {}", self))
                }
            },
            TextType::Fixed(v) => {
                if self.len() > *v as usize {
                    return Err(format!("Invalid fixed text length {}, should be {}", self, v))
                }
            },
            TextType::Variable => { todo!("variable text validation") }
        }

        Ok(())
    }
}

impl Validate for NumericValue {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        let data_type = match ct {
            ColumnType::Numeric(t) => t,
            _ => return Err(format!("Column '{}' is not a numeric type", c.name))
        };

        match (self, data_type) {
            (NumericValue::IntU8(_), NumericType::IntU8) => Ok(()),
            (NumericValue::IntU16(_), NumericType::IntU16) => Ok(()),
            (NumericValue::IntU32(_), NumericType::IntU32) => Ok(()),
            (NumericValue::IntU64(_), NumericType::IntU64) => Ok(()),

            (NumericValue::IntI8(_), NumericType::IntI8) => Ok(()),
            (NumericValue::IntI16(_), NumericType::IntI16) => Ok(()),
            (NumericValue::IntI32(_), NumericType::IntI32) => Ok(()),
            (NumericValue::IntI64(_), NumericType::IntI64) => Ok(()),

            (NumericValue::Float32(_), NumericType::Float32) => Ok(()),
            (NumericValue::Float64(_), NumericType::Float64) => Ok(()),

            // TODO: import value.to_type()
            _ => Err(format!("Invalid numeric value for column '{}', expected: {:?}, got: {:?}", 
                    c.name, data_type, self))
        }
    }
}

impl Validate for TimestampValue {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        let data_type = match ct {
            ColumnType::Timestamp(t) => t,
            _ => return Err(format!("Column '{}' is not a timestamp type", c.name))
        };

        match (self, data_type) {
            (TimestampValue::Seconds(_), TimestampType::Seconds) => Ok(()),
            (TimestampValue::Milliseconds(_), TimestampType::Milliseconds) => Ok(()),
            (TimestampValue::Microseconds(_), TimestampType::Microseconds) => Ok(()),
            (TimestampValue::Nanoseconds(_), TimestampType::Nanoseconds) => Ok(()),
 
            // TODO: import value.to_type()
            _ => Err(format!("Invalid timestamp value for column '{}', expected: {:?}, got: {:?}", 
                    c.name, data_type, self))
        }
    }
}

impl Validate for bool {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        if let ColumnType::Boolean = ct {
            Ok(())
        } else {
            // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: Boolean, got: {:?}", 
                    c.name, self)) 
        }
    }
}

impl Validate for Vec<u8> {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        if let ColumnType::Binary = ct {
            Ok(())
        } else {
            // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: Binary, got: {:?}", 
                    c.name, self)) 
        }
    }
}

impl Validate for Vec<Value> {
    fn validate(&self, c: &Column, ct: &ColumnType) -> Result<(), String> {
        if let ColumnType::Array(t) = &ct {
            for value in self {
                value.validate(c, t)?;
            }

            Ok(())
        } else {
             // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: Array, got: {:?}", 
                    c.name, self)) 
        } 
    }
}
