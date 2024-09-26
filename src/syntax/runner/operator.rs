use crate::{syntax::ast::{Operator, Node, Literal}, basics::row::{Value, NumericValue}};

use super::{Runner, Ctx};

impl Runner {
    pub(super) fn eval_unary(&self, operator: &Operator, right_node: &Box<Node>, ctx: &Ctx) -> Result<Option<Value>, String> {
        let right = self.run(right_node, ctx)?.ok_or("Invalid unary statement")?;

        match operator {
            Operator::Not => match right {
                Value::Boolean(value) => Ok(Some(Value::Boolean(!value))),
                _ => Err("Invalid unary not operator".to_string())
            },
            Operator::Sub => match right {
                Value::Numeric(number) => match number {
                    NumericValue::IntI64(value) => Ok(Some(Value::Numeric(NumericValue::IntI64(-value)))),
                    NumericValue::IntU64(value) => Ok(Some(Value::Numeric(NumericValue::IntI64(-(value as i64))))),
                    NumericValue::Float64(value) => Ok(Some(Value::Numeric(NumericValue::Float64(-value)))),
                    _ => unimplemented!("unary sub operator {:?}", number)
                },
                Value::Null => Ok(Some(Value::Null)),
                _ => Err("Invalid unary sub operator".to_string())
            },
            Operator::Inc => match (&right, &**right_node) {
                (Value::Numeric(_), Node::Literal(Literal::Identifier(ref identifier))) => {
                    let value = self.eval_add(&right, &Value::Numeric(NumericValue::IntU64(1)))?.ok_or("eval_add returned None")?;
                    return self.eval_assignment(identifier, &Node::Value(value), ctx)
                },
                _ => Err("Invalid unary inc left-hand side".to_string())
            },
            Operator::Dec => match (&right, &**right_node) {
                (Value::Numeric(_), Node::Literal(Literal::Identifier(ref identifier))) => {
                    let value = self.eval_sub(&right, &Value::Numeric(NumericValue::IntU64(1)))?.ok_or("eval_sub returned None")?;
                    return self.eval_assignment(identifier, &Node::Value(value), ctx)
                },
                _ => Err("Invalid unary dec left-hand side".to_string())
            },
            Operator::BitNot => match right {
                Value::Numeric(number) => match number {
                    NumericValue::IntI64(value) => Ok(Some(Value::Numeric(NumericValue::IntI64(!value)))),
                    NumericValue::IntU64(value) => Ok(Some(Value::Numeric(NumericValue::IntI64(!(value as i64))))),
                    _ => Err("Invalid bitwise not number".to_string())
                },
                Value::Null => Ok(Some(Value::Null)),
                _ => Err("Invalid bitwise not operator".to_string())
            },
            _ => Err("Invalid unary operator".to_string())
        }
    }

    pub(super) fn eval_binary(&self, left: &Box<Node>, operator: &Operator, right: &Box<Node>, ctx: &Ctx) -> Result<Option<Value>, String> {
        let left = self.run(left, ctx)?.ok_or("Invalid left-hand side".to_string())?;
        let right = self.run(right, ctx)?.ok_or("Invalid right-hand side".to_string())?;

        match operator {
            Operator::Add => self.eval_add(&left, &right),
            Operator::Sub => self.eval_sub(&left, &right),
            Operator::Mul => self.eval_mul(&left, &right),
            Operator::Div => self.eval_div(&left, &right),
            Operator::Mod => self.eval_mod(&left, &right),
            Operator::Pow => self.eval_pow(&left, &right),
            
            Operator::Eq => Ok(Some(Value::Boolean(left == right))),
            Operator::Ne => Ok(Some(Value::Boolean(left != right))),
            Operator::Lt => Ok(Some(Value::Boolean(left < right))),
            Operator::Le => Ok(Some(Value::Boolean(left <= right))),
            Operator::Gt => Ok(Some(Value::Boolean(left > right))),
            Operator::Ge => Ok(Some(Value::Boolean(left >= right))),
            
            Operator::And => self.eval_and(&left, &right),
            Operator::Or => self.eval_or(&left, &right),

            _ => Err("Invalid binary operator".to_string())
        }
    }
}
