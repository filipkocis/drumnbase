use std::fmt::Display;

use crate::basics::{Value, value::{TimestampValue, NumericValue}};

use super::{ColumnType, Column, TimestampType, NumericType, TextType};

pub trait Transform {
    /// Transforms a Value to the column's data type if possible and needed
    /// Returns the transformed or original value, or an error if the transformation is invalid
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String>;
}

impl Column {
    /// Validates a Value against the column's data type
    /// It checks if the value equals the column's data type
    pub fn transform_value(&self, value: &Value) -> Result<Value, String> {
        value.transform(self, &self.data_type)
    }
}

impl Transform for Value {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
        match self {
            Value::Text(v) => v.transform(c, ct),
            Value::Numeric(v) => v.transform(c, ct),
            Value::Timestamp(v) => v.transform(c, ct),
            Value::Boolean(v) => v.transform(c, ct),
            Value::Binary(v) => v.transform(c, ct),
            Value::Array(v) => v.transform(c, ct),
            Value::Enum(v) => v.transform(c, ct),
            Value::UUID(v) => v.transform(c, ct),
            Value::Null => {
                if c.not_null {
                    return Err(format!("Column '{}' does not allow NULL values", c.name))
                }

                Ok(self.clone())
            }
        }
    }
}

impl Transform for str {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
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
        
        Ok(Value::Text(self.to_string()))
    }
}

fn convert_number<T, U>(n: T, nt: &NumericType, nv: &NumericValue) -> Result<U, String> where T: TryInto<U> + Display + Copy, {
    n.try_into().or(Err(format!("Overflow for numeric value '{}', expected: {:?}, got: {:?}", n, nt, nv)))
}

impl Transform for NumericValue {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
        let data_type = match ct {
            ColumnType::Numeric(t) => t,
            _ => return Err(format!("Column '{}' is not a numeric type, expected {:?}, got {:?}", c.name, ct, self))
        };


        let numeric_value = match (self, data_type) {
            (NumericValue::IntU8(n), NumericType::IntU8) => NumericValue::IntU8(*n),
            (NumericValue::IntU8(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntU8(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::Float32) => NumericValue::Float32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU8(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntU16(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::IntU16) => NumericValue::IntU16(*n),
            (NumericValue::IntU16(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntU16(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::Float32) => NumericValue::Float32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU16(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntU32(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::IntU32) => NumericValue::IntU32(*n),
            (NumericValue::IntU32(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntU32(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntU32(n), NumericType::Float32) => NumericValue::Float32(*n as f32),
            (NumericValue::IntU32(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntU64(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntU64) => NumericValue::IntU64(*n), 
            (NumericValue::IntU64(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntU64(n), NumericType::Float32) => NumericValue::Float32(*n as f32),
            (NumericValue::IntU64(n), NumericType::Float64) => NumericValue::Float64(*n as f64),



            (NumericValue::IntI8(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntI8(n), NumericType::IntI8) => NumericValue::IntI8(*n),
            (NumericValue::IntI8(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::Float32) => NumericValue::Float32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI8(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntI16(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntI16(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::IntI16) => NumericValue::IntI16(*n),
            (NumericValue::IntI16(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::Float32) => NumericValue::Float32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI16(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntI32(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntI32(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::IntI32) => NumericValue::IntI32(*n),
            (NumericValue::IntI32(n), NumericType::IntI64) => NumericValue::IntI64(convert_number(*n, data_type, self)?),
            (NumericValue::IntI32(n), NumericType::Float32) => NumericValue::Float32(*n as f32),
            (NumericValue::IntI32(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::IntI64(n), NumericType::IntU8) => NumericValue::IntU8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntU16) => NumericValue::IntU16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntU32) => NumericValue::IntU32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntU64) => NumericValue::IntU64(convert_number(*n, data_type, self)?), 
            (NumericValue::IntI64(n), NumericType::IntI8) => NumericValue::IntI8(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntI16) => NumericValue::IntI16(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntI32) => NumericValue::IntI32(convert_number(*n, data_type, self)?),
            (NumericValue::IntI64(n), NumericType::IntI64) => NumericValue::IntI64(*n),
            (NumericValue::IntI64(n), NumericType::Float32) => NumericValue::Float32(*n as f32),
            (NumericValue::IntI64(n), NumericType::Float64) => NumericValue::Float64(*n as f64),



            (NumericValue::Float32(n), NumericType::Float32) => NumericValue::Float32(*n),
            (NumericValue::Float32(n), NumericType::Float64) => NumericValue::Float64(convert_number(*n, data_type, self)?),

            (NumericValue::Float64(n), NumericType::Float32) => NumericValue::Float32(*n as f32),
            (NumericValue::Float64(n), NumericType::Float64) => NumericValue::Float64(*n as f64),

            _ => return Err(format!("Could not transform numeric value {:?} to {:?}, required by column '{}'", self, data_type, c.name))
        };

        Ok(Value::Numeric(numeric_value))
    }
}

impl Transform for TimestampValue {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
        let data_type = match ct {
            ColumnType::Timestamp(t) => t,
            _ => return Err(format!("Column '{}' is not a timestamp type, expected {:?} got {:?}", c.name, ct, self))
        };

        let mul = |n: &u64, m| n.checked_mul(m).ok_or(format!("Overflow for timestamp value '{}', expected: {:?} got: {:?}", n, data_type, self));

        let timestamp_value = match (self, data_type) {
            (TimestampValue::Seconds(s), TimestampType::Seconds) => TimestampValue::Seconds(*s),
            (TimestampValue::Seconds(s), TimestampType::Milliseconds) => TimestampValue::Milliseconds(mul(s, 1_000)?),
            (TimestampValue::Seconds(s), TimestampType::Microseconds) => TimestampValue::Microseconds(mul(s, 1_000_000)?),
            (TimestampValue::Seconds(s), TimestampType::Nanoseconds) => TimestampValue::Nanoseconds(mul(s, 1_000_000_000)?),

            (TimestampValue::Milliseconds(s), TimestampType::Seconds) => TimestampValue::Seconds(s / 1_000),
            (TimestampValue::Milliseconds(s), TimestampType::Milliseconds) => TimestampValue::Milliseconds(*s),
            (TimestampValue::Milliseconds(s), TimestampType::Microseconds) => TimestampValue::Microseconds(mul(s, 1_000)?),
            (TimestampValue::Milliseconds(s), TimestampType::Nanoseconds) => TimestampValue::Nanoseconds(mul(s, 1_000_000)?),

            (TimestampValue::Microseconds(s), TimestampType::Seconds) => TimestampValue::Seconds(s / 1_000_000),
            (TimestampValue::Microseconds(s), TimestampType::Milliseconds) => TimestampValue::Milliseconds(s / 1_000),
            (TimestampValue::Microseconds(s), TimestampType::Microseconds) => TimestampValue::Microseconds(*s),
            (TimestampValue::Microseconds(s), TimestampType::Nanoseconds) => TimestampValue::Nanoseconds(mul(s, 1_000)?),

            (TimestampValue::Nanoseconds(s), TimestampType::Seconds) => TimestampValue::Seconds(s / 1_000_000_000),
            (TimestampValue::Nanoseconds(s), TimestampType::Milliseconds) => TimestampValue::Milliseconds(s / 1_000_000),
            (TimestampValue::Nanoseconds(s), TimestampType::Microseconds) => TimestampValue::Microseconds(s / 1_000),
            (TimestampValue::Nanoseconds(s), TimestampType::Nanoseconds) => TimestampValue::Nanoseconds(*s),
        };

        Ok(Value::Timestamp(timestamp_value))
    }
}

impl Transform for bool {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
        if let ColumnType::Boolean = ct {
            Ok(Value::Boolean(*self))
        } else {
            // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: {:?}, got: Boolean", 
                    c.name, ct)) 
        }
    }
}

impl Transform for Vec<u8> {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> { 
        if let ColumnType::Binary = ct {
            Ok(Value::Binary(self.clone()))
        } else {
            // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: {:?}, got: Binary", 
                    c.name, ct)) 
        }
    }
}

impl Transform for Vec<Value> {
    fn transform(&self, c: &Column, ct: &ColumnType) -> Result<Value, String> {
        if let ColumnType::Array(t) = &ct {
            let mut transformed = Vec::with_capacity(self.len());

            for value in self {
                transformed.push(value.transform(c, t)?);
            }

            Ok(Value::Array(transformed))
        } else {
             // TODO: import value.to_type()
            return Err(format!("Invalid value for column '{}', expected: {:?}, got: Array", 
                    c.name, ct)) 
        } 
    }
}
