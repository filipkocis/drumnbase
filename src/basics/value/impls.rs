use std::fmt::Display;

use super::{NumericValue, TimestampValue, Value};

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

impl Display for TimestampValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimestampValue::Seconds(t) => write!(f, "{}", t),
            TimestampValue::Milliseconds(t) => write!(f, "{}", t),
            TimestampValue::Microseconds(t) => write!(f, "{}", t),
            TimestampValue::Nanoseconds(t) => write!(f, "{}", t),
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
            Value::Array(s) => write!(f, "[{}]", s.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", ")),
            Value::Enum(s) => write!(f, "{}", s),
            Value::UUID(s) => write!(f, "{}", s),
            Value::Null => write!(f, "NULL"),
        }
    }
}

impl PartialOrd for NumericValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumericValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_f64 = self.to_f64();
        let other_f64 = other.to_f64();

        self_f64.partial_cmp(&other_f64).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialEq for NumericValue {
    fn eq(&self, other: &Self) -> bool {
        let self_f64 = self.to_f64();
        let other_f64 = other.to_f64();

        self_f64 == other_f64
    }
}

impl Eq for NumericValue { }

impl NumericValue {
    pub fn to_f64(&self) -> f64 {
        match *self {
            NumericValue::IntU8(v) => v as f64,
            NumericValue::IntU16(v) => v as f64,
            NumericValue::IntU32(v) => v as f64,
            NumericValue::IntU64(v) => v as f64,

            NumericValue::IntI8(v) => v as f64,
            NumericValue::IntI16(v) => v as f64,
            NumericValue::IntI32(v) => v as f64,
            NumericValue::IntI64(v) => v as f64,

            NumericValue::Float32(v) => v as f64,
            NumericValue::Float64(v) => v,
        }
    }

    pub fn to_i128(&self) -> i128 {
        match *self {
            NumericValue::IntU8(v) => v as i128,
            NumericValue::IntU16(v) => v as i128,
            NumericValue::IntU32(v) => v as i128,
            NumericValue::IntU64(v) => v as i128,

            NumericValue::IntI8(v) => v as i128,
            NumericValue::IntI16(v) => v as i128,
            NumericValue::IntI32(v) => v as i128,
            NumericValue::IntI64(v) => v as i128,

            NumericValue::Float32(v) => v as i128,
            NumericValue::Float64(v) => v as i128,
        }
    }
}
