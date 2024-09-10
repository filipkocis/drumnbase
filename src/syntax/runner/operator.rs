use crate::{syntax::ast::{Operator, Node, Literal}, basics::row::{Value, NumericValue}};

use super::Runner;

impl Runner {
    pub(super) fn eval_unary(&self, operator: &Operator, right_node: &Box<Node>) -> Result<Value, String> {
        let right = self.run(right_node)?;

        match operator {
            Operator::Not => match right {
                Value::Boolean(value) => Ok(Value::Boolean(!value)),
                _ => Err("Invalid unary not operator".to_string())
            },
            Operator::Sub => match right {
                Value::Numeric(number) => match number {
                    NumericValue::IntI64(value) => Ok(Value::Numeric(NumericValue::IntI64(-value))),
                    NumericValue::IntU64(value) => Ok(Value::Numeric(NumericValue::IntI64(-(value as i64)))),
                    NumericValue::Float64(value) => Ok(Value::Numeric(NumericValue::Float64(-value))),
                    _ => unimplemented!("unary sub operator {:?}", number)
                },
                Value::Null => Ok(Value::Null),
                _ => Err("Invalid unary sub operator".to_string())
            },
            Operator::Inc => match (&right, &**right_node) {
                (Value::Numeric(_), Node::Literal(Literal::Identifier(ref identifier))) => {
                    let value = self.eval_add(&right, &Value::Numeric(NumericValue::IntU64(1)))?;
                    return self.eval_assignment(identifier, &Node::Value(value))
                },
                _ => Err("Invalid unary inc left-hand side".to_string())
            },
            Operator::Dec => match (&right, &**right_node) {
                (Value::Numeric(_), Node::Literal(Literal::Identifier(ref identifier))) => {
                    let value = self.eval_sub(&right, &Value::Numeric(NumericValue::IntU64(1)))?;
                    return self.eval_assignment(identifier, &Node::Value(value))
                },
                _ => Err("Invalid unary dec left-hand side".to_string())
            },
            Operator::BitNot => match right {
                Value::Numeric(number) => match number {
                    NumericValue::IntI64(value) => Ok(Value::Numeric(NumericValue::IntI64(!value))),
                    NumericValue::IntU64(value) => Ok(Value::Numeric(NumericValue::IntI64(!(value as i64)))),
                    _ => Err("Invalid bitwise not number".to_string())
                },
                Value::Null => Ok(Value::Null),
                _ => Err("Invalid bitwise not operator".to_string())
            },
            _ => Err("Invalid unary operator".to_string())
        }
    }

    pub(super) fn eval_binary(&self, left: &Box<Node>, operator: &Operator, right: &Box<Node>) -> Result<Value, String> {
        let left = self.run(left)?;
        let right = self.run(right)?;

        match operator {
            Operator::Add => self.eval_add(&left, &right),
            Operator::Sub => self.eval_sub(&left, &right),
            Operator::Mul => self.eval_mul(&left, &right),
            Operator::Div => self.eval_div(&left, &right),
            Operator::Mod => self.eval_mod(&left, &right),
            Operator::Pow => self.eval_pow(&left, &right),
            
            Operator::Eq => Ok(Value::Boolean(left == right)),
            Operator::Ne => Ok(Value::Boolean(left != right)),
            Operator::Lt => Ok(Value::Boolean(left < right)),
            Operator::Le => Ok(Value::Boolean(left <= right)),
            Operator::Gt => Ok(Value::Boolean(left > right)),
            Operator::Ge => Ok(Value::Boolean(left >= right)),
            
            Operator::And => self.eval_and(&left, &right),
            Operator::Or => self.eval_or(&left, &right),

            _ => Err("Invalid binary operator".to_string())
        }
    }

}
