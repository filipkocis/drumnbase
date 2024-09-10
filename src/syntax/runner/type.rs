use crate::{basics::row::{Value, NumericValue}, syntax::ast::Type};

use super::Runner;

impl Runner {
    pub(crate) fn check_type(&self, type_: &Type, value: &Value) -> bool {
        match (type_, value) {
            (Type::Int, Value::Numeric(NumericValue::IntI64(_))) => true,
            (Type::UInt, Value::Numeric(NumericValue::IntU64(_))) => true,
            (Type::Float, Value::Numeric(NumericValue::Float64(_))) => true,
            (Type::String, Value::Text(_)) => true,
            (Type::Boolean, Value::Boolean(_)) => true,
            (Type::Array(type_), Value::Array(values)) => values.iter().all(|value| self.check_type(type_, value)),
            (Type::Void, Value::Null) => true,
            _ => false
        }
    }

    pub(crate) fn get_type(&self, value: &Value) -> Type {
        match value {
            Value::Numeric(NumericValue::IntI64(_)) => Type::Int,
            Value::Numeric(NumericValue::IntU64(_)) => Type::UInt,
            Value::Numeric(NumericValue::Float64(_)) => Type::Float,
            Value::Text(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Array(values) => {
                let mut types = values.iter().map(|value| self.get_type(value)).collect::<Vec<Type>>();
                types.sort();
                types.dedup();

                if types.len() == 1 {
                    Type::Array(Box::new(types[0].clone()))
                } else {
                    Type::Array(Box::new(Type::Void))
                }
            },
            Value::Null => Type::Void,

            _ => unimplemented!("get_type {:?}", value)
        }
    }
}
