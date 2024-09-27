use crate::basics::row::NULL_BYTE;

use super::{Value, NumericValue, TimestampValue};

pub trait ToBytes {
    fn to_bytes(&self, length: u32) -> Vec<u8>;
}

impl ToBytes for Value {
    fn to_bytes(&self, length: u32) -> Vec<u8> {
        match self {
            Value::Text(s) => s.to_bytes(length),
            Value::Numeric(n) => n.to_bytes(length),
            Value::Timestamp(t) => t.to_bytes(length),
            Value::Boolean(b) => b.to_bytes(length),
            Value::Binary(b) => b.clone(), 
            Value::Array(a) => a.iter().flat_map(|v| v.to_bytes(length)).collect(),
            Value::Enum(e) => e.to_bytes(length), 
            Value::UUID(u) => u.as_bytes().to_vec(), 

            Value::Null => vec![NULL_BYTE; length as usize] 
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

impl ToBytes for TimestampValue {
    fn to_bytes(&self, _: u32) -> Vec<u8> {
        match self {
            TimestampValue::Seconds(n) => n.to_be_bytes().to_vec(),
            TimestampValue::Milliseconds(n) => n.to_be_bytes().to_vec(),
            TimestampValue::Microseconds(n) => n.to_be_bytes().to_vec(),
            TimestampValue::Nanoseconds(n) => n.to_be_bytes().to_vec(),
        }
    }
}

impl ToBytes for bool {
    fn to_bytes(&self, _: u32) -> Vec<u8> {
        vec![*self as u8]
    }
}
