use crate::{syntax::ast::{Expression, Node}, basics::Value};

use super::{Runner, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_expression(&self, expression: &Expression, ctx: &Ctx) -> RunnerResult {
        match expression {
            Expression::Binary { left, operator, right }
                => self.eval_binary(left, operator, right, ctx), 
            Expression::Unary { operator, right } => self.eval_unary(operator, right, ctx),
            Expression::Call { name, arguments } => self.eval_call(name, arguments, ctx),
            Expression::Literal(value) => self.eval_literal(value, ctx),
            Expression::Index { object, index } => self.eval_index(object, index, ctx),

            _ => unimplemented!("expression")
        }
    }

    fn eval_index(&self, object: &Box<Node>, index: &Box<Node>, ctx: &Ctx) -> RunnerResult {
        let object = self.run(object, ctx)?.ok_or("Object cannot be a statement with no return value")?;
        let index = self.run(index, ctx)?.ok_or("Index cannot be a statement with no return value")?;

        if let Value::Array(array) = object {
            let index = match index {
                Value::Numeric(numeric) => numeric.to_i128(), 
                _ => return Err("Invalid index".to_string())
            };

            if index < 0 || index >= array.len() as i128 || index >= usize::MAX as i128 {
                return Err(format!("Index '{}' out of bounds", index))
            }

            let value = array.into_iter().nth(index as usize).unwrap();
            return Ok(Some(value))
        }

        Err(format!("Object of type {:?} is not indexable", self.get_type(&object)))
    }
}
