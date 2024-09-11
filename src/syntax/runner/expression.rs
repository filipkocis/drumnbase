use crate::{syntax::ast::{Expression, Node}, basics::row::{Value, NumericValue}};

use super::Runner;

impl Runner {
    pub(super) fn eval_expression(&self, expression: &Expression) -> Result<Option<Value>, String> {
        match expression {
            Expression::Binary { left, operator, right }
                => self.eval_binary(left, operator, right), 
            Expression::Unary { operator, right } => self.eval_unary(operator, right),
            Expression::Call { name, arguments } => self.eval_call(name, arguments),
            Expression::Literal(value) => self.eval_literal(value),
            Expression::Index { name, index } => self.eval_index(name, index),

            _ => unimplemented!("expression")
        }
    }

    fn eval_index(&self, name: &str, index: &Box<Node>) -> Result<Option<Value>, String> {
        let index = self.run(index)?;
        let variables = self.variables.borrow();
        if let Some(Value::Array(array)) = variables.get(name) {
            if let Value::Numeric(NumericValue::IntU64(index)) = index.ok_or("Index cannot be a statement")? {
                if let Some(value) = array.get(index as usize) {
                    return Ok(Some(value.clone()))
                }
            }
        }

        Err("Invalid index".to_string())
    }
}
