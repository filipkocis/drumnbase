use std::{fmt::Display, array::TryFromSliceError, string::FromUtf8Error};

use super::column::{Column, ColumnType, NumericType, TextType};

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" | ");
        write!(f, "{}", string)
    }
}

impl Display for NumericValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericValue::IntI8(n) => write!(f, "{}", n),
            NumericValue::IntI16(n) => write!(f, "{}", n),
            NumericValue::IntI32(n) => write!(f, "{}", n),
            NumericValue::IntI64(n) => write!(f, "{}", n),
            NumericValue::IntU8(n) => write!(f, "{}", n),
            NumericValue::IntU16(n) => write!(f, "{}", n),
            NumericValue::IntU32(n) => write!(f, "{}", n),
            NumericValue::IntU64(n) => write!(f, "{}", n),
            NumericValue::Float32(n) => write!(f, "{}", n),
            NumericValue::Float64(n) => write!(f, "{}", n),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Text(s) => write!(f, "{}", s),
            Value::Numeric(n) => write!(f, "{}", n),
            Value::Timestamp(s) => write!(f, "{}", s),
            Value::Boolean(s) => write!(f, "{}", s),
            Value::Binary(s) => write!(f, "{:?}", s),
            Value::Array(s) => write!(f, "{:?}", s),
            Value::Enum(s) => write!(f, "{}", s),
            Value::UUID(s) => write!(f, "{}", s),
            Value::Null => write!(f, "NULL"),
        }
    }
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NumericValue {
    IntU8(u8),
    IntU16(u16),
    IntU32(u32),
    IntU64(u64),

    IntI8(i8),
    IntI16(i16),
    IntI32(i32),
    IntI64(i64),

    Float32(f32),
    Float64(f64),

}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Text(String),
    Numeric(NumericValue),
    Timestamp(i64),
    Boolean(bool),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Enum(String),
    UUID(String),
    Null,
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn like(&self, value: &Value) -> bool {
        match (self, value) {
            (Value::Text(s1), Value::Text(s2)) => s1.contains(s2),
            _ => false,
        }
    }

    pub fn in_(&self, value: &Value) -> bool {
        match (self, value) {
            (Value::Array(a), Value::Array(b)) => a.iter().all(|v| b.contains(v)),
            (a, Value::Array(b)) => b.contains(a),
            _ => false,
        }
    }

    pub fn between(&self, value: &Value) -> bool {
        match (self, value) {
            (Value::Numeric(n), Value::Array(a)) => {
                
                true
            },
            _ => false,
        };

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    pub fn new() -> Row {
        Row {
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn set(&mut self, index: usize, value: Value) {
        if index >= self.values.len() {
            self.values.resize(index + 1, Value::Null);
        }

        self.values[index] = value;
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index >= self.values.len() { return None; }
        Some(self.values.remove(index))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.values.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Value> {
        self.values.iter()
    }

    pub fn convert_to_bytes(&self, columns: &Vec<Column>) -> Vec<u8> {
        self.values.iter().enumerate().flat_map(|(i, v)| {
            let length = columns[i].length;
            v.to_bytes(length)
        }).collect()
    }

    pub fn convert_from_bytes(bytes: &[u8], columns: &Vec<Column>) -> Result<Self, String> {
        let mut row = Row::new();
        let mut offset = 0;

        for column in columns {
            let length = column.length as usize;
            let value = Value::from_bytes(&bytes[offset..offset + length], &column.data_type)?;
            row.add(value);
            offset += length;
        }
        
        Ok(row)
    }

    pub fn with_excluded_columns(&self, indexes: &[usize]) -> Row {
        let mut row = Row::new();
        self.values.iter().enumerate().for_each(|(i, v)| {
            if !indexes.contains(&i) {
                row.add(v.clone());
            }
        });
        row
    }

    /// Returns a new row with only the columns at the given indexes.
    /// Panics if any of the indexes are out of bounds
    pub fn with_kept_columns(&self, indexes: &[usize]) -> Row {
        let mut row = Row::new();
        indexes.iter().for_each(|&i| {
            row.add(self.values[i].clone());
        });
        row
    }
}

trait ToBytes {
    fn to_bytes(&self, length: u32) -> Vec<u8>;
}

impl ToBytes for Value {
    fn to_bytes(&self, length: u32) -> Vec<u8> {
        match self {
            Value::Text(s) => s.to_bytes(length),
            Value::Numeric(n) => n.to_bytes(length),
            Value::Timestamp(t) => t.to_be_bytes().to_vec(),
            Value::Boolean(b) => b.to_bytes(length),
            Value::Binary(b) => b.clone(), 
            Value::Array(a) => a.iter().flat_map(|v| v.to_bytes(length)).collect(),
            Value::Enum(e) => e.to_bytes(length), 
            Value::UUID(u) => u.as_bytes().to_vec(), 

            Value::Null => vec![0; length as usize] 
        }
    }
}

impl ToBytes for String {
    fn to_bytes(&self, length: u32) -> Vec<u8> {
        let mut bytes = self.as_bytes().to_vec();
        bytes.resize(length as usize, 0);
        bytes 
    }
}

impl ToBytes for NumericValue {
    fn to_bytes(&self, _: u32) -> Vec<u8> {
        match self {
            NumericValue::IntI8(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntI16(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntI32(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntI64(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntU8(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntU16(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntU32(n) => n.to_be_bytes().to_vec(),
            NumericValue::IntU64(n) => n.to_be_bytes().to_vec(),
            NumericValue::Float32(n) => n.to_be_bytes().to_vec(),
            NumericValue::Float64(n) => n.to_be_bytes().to_vec(),
        }
    }
}

impl ToBytes for bool {
    fn to_bytes(&self, _: u32) -> Vec<u8> {
        vec![*self as u8]
    }
}

trait FromBytes {
    type EnumType;
    fn from_bytes(bytes: &[u8], enum_type: &Self::EnumType) -> Result<Self, String> where Self: Sized;
}

impl FromBytes for Value {
    type EnumType = ColumnType;

    fn from_bytes(bytes: &[u8], column_type: &ColumnType) -> Result<Self, String> {
        let value = match column_type {
            ColumnType::Text(text_type) => {
                let map_err = |e: FromUtf8Error| e.to_string();
                let text = match text_type {
                    TextType::Char => String::from_utf8(bytes.to_vec()).map_err(map_err)?,
                    // TextType::Variable => {
                    //     let s = String::from_utf8(bytes.to_vec()).map_err(map_err)?;
                    //     Value::Text(s)
                    // },
                    TextType::Fixed(length) => {
                        let mut bytes = bytes.to_vec();
                        bytes.truncate(*length as usize);
                        String::from_utf8(bytes).map_err(map_err)?
                    }
                    
                    _ => todo!()
                };

                Value::Text(text)
            },
            ColumnType::Numeric(numeric_type) => {
                let v = NumericValue::from_bytes(bytes, numeric_type)?;
                Value::Numeric(v)
            },

            _ => todo!()
        };

        Ok(value)
    }
}

impl FromBytes for NumericValue {
    type EnumType = NumericType;

    fn from_bytes(bytes: &[u8], numeric_type: &NumericType) -> Result<Self, String> where Self: Sized {
        let map_err = |e: TryFromSliceError| e.to_string(); 

        let numeric_value = match numeric_type {
            NumericType::IntI8 => NumericValue::IntI8(i8::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI16 => NumericValue::IntI16(i16::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI32 => NumericValue::IntI32(i32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntI64 => NumericValue::IntI64(i64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU8 => NumericValue::IntU8(u8::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU16 => NumericValue::IntU16(u16::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU32 => NumericValue::IntU32(u32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::IntU64 => NumericValue::IntU64(u64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::Float32 => NumericValue::Float32(f32::from_be_bytes(bytes.try_into().map_err(map_err)?)),
            NumericType::Float64 => NumericValue::Float64(f64::from_be_bytes(bytes.try_into().map_err(map_err)?)),
        };

        Ok(numeric_value)
    }
}
