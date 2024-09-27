use crate::{syntax::{ast::{Literal, Number, Node}, context::RunnerContextVariable}, basics::{Value, value::NumericValue}};

use super::{Runner, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_literal(&self, literal: &Literal, ctx: &Ctx) -> RunnerResult {
        match literal {
            Literal::Identifier(name) => self.eval_identifier(name, ctx),
            Literal::Number(number) => self.eval_number(number),
            Literal::String(value) => Ok(Some(Value::Text(value.clone()))),
            Literal::Boolean(value) => Ok(Some(Value::Boolean(*value))),
            Literal::Array(values) => self.eval_array(values, ctx),
            Literal::Null => Ok(Some(Value::Null))
        }
    }

    fn eval_identifier(&self, name: &str, ctx: &Ctx) -> RunnerResult {
        // TODO: find a way to return a reference without cloning
        let value = ctx.get(name)?;
        let value = value.borrow().clone().into_owned();
        
        Ok(Some(value))
    }

    fn eval_number(&self, number: &Number) -> RunnerResult {
        let numeric = match number {
            Number::Int(value) => NumericValue::IntI64(*value),
            Number::UInt(value) => NumericValue::IntU64(*value),
            Number::Float(value) => NumericValue::Float64(*value),
        };

        Ok(Some(Value::Numeric(numeric)))
    }

    fn eval_array(&self, values: &Vec<Node>, ctx: &Ctx) -> RunnerResult {
        let mut result = Vec::new();
        for value in values {
            let value = self.run(&value, ctx)?;
            match value {
                Some(value) => result.push(value),
                None => Err(format!("Invalid array element: {:?}", value))?
            }
        } 

        Ok(Some(Value::Array(result)))
    }
}
