use std::fmt::Display;

impl ToString for Row {
    fn to_string(&self) -> String {
        self.values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" | ")
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


#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

pub trait ToBytes {
    fn to_bytes(&self, length: u32) -> Vec<u8>;
}

impl ToBytes for Row {
    fn to_bytes(&self, length: u32) -> Vec<u8> {
        self.values.iter().flat_map(|v| v.to_bytes(length)).collect()
    }
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
