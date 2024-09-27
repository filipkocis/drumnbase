use crate::{syntax::{ast::{Expression, Node}, context::RunnerContextVariable}, basics::{Value, value::NumericValue}};

use super::{Runner, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_expression(&self, expression: &Expression, ctx: &Ctx) -> RunnerResult {
        match expression {
            Expression::Binary { left, operator, right }
                => self.eval_binary(left, operator, right, ctx), 
            Expression::Unary { operator, right } => self.eval_unary(operator, right, ctx),
            Expression::Call { name, arguments } => self.eval_call(name, arguments, ctx),
            Expression::Literal(value) => self.eval_literal(value, ctx),
            Expression::Index { name, index } => self.eval_index(name, index, ctx),

            _ => unimplemented!("expression")
        }
    }

    fn eval_index(&self, name: &str, index: &Box<Node>, ctx: &Ctx) -> RunnerResult {
        let index = self.run(index, ctx)?.ok_or("Index cannot be a statement with no return value")?;

        if let Value::Array(array) = ctx.get(name)?.borrow().as_ref() {
            // TODO: Implement index number validation
            if let Value::Numeric(NumericValue::IntU64(index)) = index {
                return match array.get(index as usize) {
                    // TODO: Implement way to skip cloning when needing a reference
                    Some(value) => Ok(Some(value.clone())),
                    None => Err(format!("Index '{}' out of bounds", index))
                }
            }
        }

        Err("Invalid index".to_string())
    }
}
