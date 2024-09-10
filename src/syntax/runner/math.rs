use crate::basics::row::{Value, NumericValue};

use super::Runner;

impl Runner {
    pub(super) fn eval_add(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left + right))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left + *right as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 + right))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64(left + right))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(*left as i64 + right))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 + right))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left + right))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left + *right as f64))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left + *right as f64))),

                _ => unimplemented!("add numeric {:?}", (left, right))
            },
            (Value::Text(left), Value::Text(right)) => Ok(Value::Text(format!("{}{}", left, right))),
            (Value::Array(left), Value::Array(right)) => {
                let mut result = left.clone();
                result.extend(right.clone());
                Ok(Value::Array(result))
            },
            (Value::Null, _) => Ok(right.clone()),
            (_, Value::Null) => Ok(left.clone()),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left || *right)),

            _ => unimplemented!("add {:?}", (left, right))
        }
    }

    pub(super) fn eval_sub(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left - right))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left - *right as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 - right))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64(left - right))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(*left as i64 - right))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 - right))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left - right))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left - *right as f64))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left - *right as f64))),

                _ => unimplemented!("sub numeric {:?}", (left, right))
            },
            (Value::Null, _) => Ok(self.eval_sub(&Value::Numeric(NumericValue::IntI64(0)), right)?),
            (_, Value::Null) => Ok(left.clone()),
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left && !*right)),

            _ => unimplemented!("sub {:?}", (left, right))
        }
    }

    pub(super) fn eval_mul(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left * right))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left * *right as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 * right))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64(left * right))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(*left as i64 * right))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 * right))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left * right))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left * *right as f64))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left * *right as f64))),

                _ => unimplemented!("mul numeric {:?}", (left, right))
            },
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("mul {:?}", (left, right))
        }
    }

    pub(super) fn eval_div(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left / right))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left / *right as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 / right))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64(left / right))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(*left as i64 / right))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 / right))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left / right))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left / *right as f64))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left / *right as f64))),

                _ => unimplemented!("div numeric {:?}", (left, right))
            },
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("div {:?}", (left, right))
        }
    }

    pub(super) fn eval_mod(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left % right))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64(left % *right as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 % right))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64(left % right))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64(*left as i64 % right))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(*left as f64 % right))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left % right))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left % *right as f64))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left % *right as f64))),

                _ => unimplemented!("mod numeric {:?}", (left, right))
            },
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("mod {:?}", (left, right))
        }
    }

    pub(super) fn eval_pow(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Numeric(left), Value::Numeric(right)) => match (left, right) {
                (NumericValue::IntI64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64((*left as f64).powf(*right as f64) as i64))),
                (NumericValue::IntI64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntI64((*left as f64).powf(*right as f64) as i64))),
                (NumericValue::IntI64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64((*left as f64).powf(*right)))),

                (NumericValue::IntU64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::IntU64((*left as f64).powf(*right as f64) as u64))),
                (NumericValue::IntU64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::IntI64((*left as f64).powf(*right as f64) as i64))),
                (NumericValue::IntU64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64((*left as f64).powf(*right)))),

                (NumericValue::Float64(left), NumericValue::Float64(right)) => Ok(Value::Numeric(NumericValue::Float64(left.powf(*right)))),
                (NumericValue::Float64(left), NumericValue::IntI64(right)) => Ok(Value::Numeric(NumericValue::Float64(left.powf(*right as f64)))),
                (NumericValue::Float64(left), NumericValue::IntU64(right)) => Ok(Value::Numeric(NumericValue::Float64(left.powf(*right as f64)))),

                _ => unimplemented!("pow numeric {:?}", (left, right))
            },
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("pow {:?}", (left, right))
        }
    }

    pub(super) fn eval_and(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left && *right)),
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("and {:?}", (left, right))
        }
    }

    pub(super) fn eval_or(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left || *right)),
            (Value::Null, _) => Ok(Value::Null),
            (_, Value::Null) => Ok(Value::Null),

            _ => unimplemented!("or {:?}", (left, right))
        }
    }
}

