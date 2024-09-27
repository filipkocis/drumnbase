mod impls;
mod from_bytes;
mod to_bytes;
mod as_inner;

pub use from_bytes::FromBytes;
pub use to_bytes::ToBytes;

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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TimestampValue {
    Seconds(u64),
    Milliseconds(u64),
    Microseconds(u64),
    Nanoseconds(u64),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Text(String),
    Numeric(NumericValue),
    Timestamp(TimestampValue),
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

    #[allow(unused_variables)]
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
