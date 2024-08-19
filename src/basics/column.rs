use std::str::FromStr;

use super::row::{Value, NumericValue, TimestampValue};

#[derive(Debug)]
pub enum TextType {
    Char,
    Variable,
    Fixed(u32),
}

#[derive(Debug)]
pub enum NumericType {
    IntU8,
    IntU16,
    IntU32,
    IntU64,

    IntI8,
    IntI16,
    IntI32,
    IntI64,

    Float32,
    Float64,
}

#[derive(Debug)]
pub enum TimestampType {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    // Date,
    // Time,
}


#[derive(Debug)]
pub enum ColumnType {
    Numeric(NumericType),
    Text(TextType),
    Timestamp(TimestampType),
    Boolean,
    Binary,
    Array,
    Enum,
    UUID,
}

impl NumericType {
    pub fn parse(&self, value: &str) -> Result<Value, String> {
        match self {
            NumericType::IntI8 => value.parse::<i8>().map(|v| Value::Numeric(NumericValue::IntI8(v))).map_err(|e| e.to_string()), 
            NumericType::IntI16 => value.parse::<i16>().map(|v| Value::Numeric(NumericValue::IntI16(v))).map_err(|e| e.to_string()), 
            NumericType::IntI32 => value.parse::<i32>().map(|v| Value::Numeric(NumericValue::IntI32(v))).map_err(|e| e.to_string()), 
            NumericType::IntI64 => value.parse::<i64>().map(|v| Value::Numeric(NumericValue::IntI64(v))).map_err(|e| e.to_string()), 
            
            NumericType::IntU8 => value.parse::<u8>().map(|v| Value::Numeric(NumericValue::IntU8(v))).map_err(|e| e.to_string()), 
            NumericType::IntU16 => value.parse::<u16>().map(|v| Value::Numeric(NumericValue::IntU16(v))).map_err(|e| e.to_string()), 
            NumericType::IntU32 => value.parse::<u32>().map(|v| Value::Numeric(NumericValue::IntU32(v))).map_err(|e| e.to_string()), 
            NumericType::IntU64 => value.parse::<u64>().map(|v| Value::Numeric(NumericValue::IntU64(v))).map_err(|e| e.to_string()), 

            NumericType::Float32 => value.parse::<f32>().map(|v| Value::Numeric(NumericValue::Float32(v))).map_err(|e| e.to_string()), 
            NumericType::Float64 => value.parse::<f64>().map(|v| Value::Numeric(NumericValue::Float64(v))).map_err(|e| e.to_string()), 
        } 
    }
}

impl TextType {
    pub fn parse(&self, value: &str) -> Result<Value, String> {
        match self {
            TextType::Char => {
                if value.len() != 1 {
                    return Err(format!("Invalid char length {}", value))
                }
            },
            TextType::Fixed(v) => {
                if value.len() > *v as usize {
                    return Err(format!("Invalid fixed text length {}, should be {}", value, v))
                }
            }
            TextType::Variable => { todo!() }
        }; 

        Ok(Value::Text(value.to_string()))
    }
}

impl TimestampType {
    pub fn parse(&self, value: &str) -> Result<Value, String> {
        match self {
            TimestampType::Seconds => value.parse::<u64>().map(|v| Value::Timestamp(TimestampValue::Seconds(v))).map_err(|e| e.to_string()),
            TimestampType::Milliseconds => value.parse::<u64>().map(|v| Value::Timestamp(TimestampValue::Milliseconds(v))).map_err(|e| e.to_string()),
            TimestampType::Microseconds => value.parse::<u64>().map(|v| Value::Timestamp(TimestampValue::Microseconds(v))).map_err(|e| e.to_string()),
            TimestampType::Nanoseconds => value.parse::<u64>().map(|v| Value::Timestamp(TimestampValue::Nanoseconds(v))).map_err(|e| e.to_string()),
        }
    }
}

impl ColumnType {
    pub fn parse(&self, value: &str) -> Result<Value, String> {
        let result = match self {
            ColumnType::Numeric(t) => t.parse(value),
            ColumnType::Text(t) => t.parse(value),
            ColumnType::Timestamp(t) => t.parse(value),

            ColumnType::Boolean => value.parse::<bool>().map(Value::Boolean).map_err(|e| e.to_string()),
            ColumnType::Binary => value.parse::<Bytes>().map(|v| Value::Binary(v.0)).map_err(|e| e.to_string()),
            ColumnType::Array => todo!(),
            ColumnType::Enum => todo!(),
            ColumnType::UUID => todo!(),
        };

        result.map_err(|e| format!("Failed to parse value '{}' as {:?}: {}", value, self, e))
    }
}

struct Bytes(Vec<u8>);
impl FromStr for Bytes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = Vec::new(); 

        for byte in s.split(",") {
            bytes.push(u8::from_str_radix(byte, 2).map_err(|e| e.to_string())?);
        }

        Ok(Bytes(bytes))
    }
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: ColumnType,
    pub length: u32,
    pub default: Option<String>,
    pub not_null: bool,
    pub unique: bool,
    pub read_only: bool,
    // pub primary_key: bool,
    // pub foreigh_key: bool,
    // pub check: bool,
    // pub references: String,
    // pub check_constraint: String,
    // pub privileges: Vec<Privilege>,
}

impl Column {
    pub fn new(name: &str, data_type: ColumnType) -> Self {
        Column {
            name: name.to_owned(),
            data_type,
            length: 0,
            default: None,
            not_null: false,
            unique: false,
            read_only: false,
        }
    }

    pub fn set_length(&mut self, length: u32) {
        self.length = length;
    }

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
