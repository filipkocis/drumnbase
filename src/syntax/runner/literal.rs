use crate::{syntax::ast::{Literal, Number, Node}, basics::row::{Value, NumericValue}};

use super::Runner;

impl Runner {
    pub(super) fn eval_literal(&self, literal: &Literal) -> Result<Option<Value>, String> {
        match literal {
            Literal::Identifier(name) => self.eval_identifier(name),
            Literal::Number(number) => self.eval_number(number),
            Literal::String(value) => Ok(Some(Value::Text(value.clone()))),
            Literal::Boolean(value) => Ok(Some(Value::Boolean(*value))),
            Literal::Array(values) => self.eval_array(values),
            Literal::Null => Ok(Some(Value::Null))
        }
    }

    fn eval_identifier(&self, name: &str) -> Result<Option<Value>, String> {
        if let Some(value) = self.variables.borrow().get(name) {
            Ok(Some(value.clone()))
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }

    fn eval_number(&self, number: &Number) -> Result<Option<Value>, String> {
        let numeric = match number {
            Number::Int(value) => NumericValue::IntI64(*value),
            Number::UInt(value) => NumericValue::IntU64(*value),
            Number::Float(value) => NumericValue::Float64(*value),
        };

        Ok(Some(Value::Numeric(numeric)))
    }

    fn eval_array(&self, values: &Vec<Node>) -> Result<Option<Value>, String> {
        let mut result = Vec::new();
        for value in values {
            let value = self.run(&value)?;
            match value {
                Some(value) => result.push(value),
                None => Err(format!("Invalid array element: {:?}", value))?
            }
        } 

        Ok(Some(Value::Array(result)))
    }
}
