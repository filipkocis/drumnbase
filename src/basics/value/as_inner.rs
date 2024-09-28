use super::{Value, NumericValue};

impl Value {
    pub fn as_text(&self) -> Option<&String> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_numeric(&self) -> Option<&NumericValue> {
        match self {
            Value::Numeric(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}
