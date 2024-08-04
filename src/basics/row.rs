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

#[derive(Debug)]
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
