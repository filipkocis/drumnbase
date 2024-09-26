use crate::{syntax::{ast::{Statement, Node}, context::RunnerContextVariable}, basics::row::Value};

use super::{Runner, Ctx};

impl Runner {
    pub(super) fn eval_statement(&self, statement: &Statement, ctx: &Ctx) -> Result<Option<Value>, String> {
        match statement {
            Statement::Assignment { name, value } => self.eval_assignment(name, value, ctx),
            Statement::Expression(_) => unimplemented!("statement expression"),
            Statement::Function { name, parameters, return_type, block } 
                => self.eval_function(name, parameters, return_type, block),     
            Statement::Let { name, value } => self.eval_declaration(name, value, ctx),
            Statement::If { condition, then_block, else_block } 
                => self.eval_if(condition, then_block, else_block, ctx),
            Statement::While { condition, block } => self.eval_while(condition, block, ctx),
            Statement::For { initializer, condition, action, block }
                => self.eval_for(initializer, condition, action, block, ctx),
            Statement::Loop { block } => self.eval_loop(block, ctx),

            Statement::Return(_) => Err("Return outside of function".to_string()),
            Statement::Break => Err("Break outside of loop".to_string()),
            Statement::Continue => Err("Continue outside of loop".to_string()),
        }
    }

    fn eval_if(&self, condition: &Node, then_block: &Box<Node>, else_block: &Option<Box<Node>>, ctx: &Ctx) -> Result<Option<Value>, String> {
        if !matches!(condition, Node::Expression(_)) {
            return Err("If condition must be an expression".to_string())
        }

        let condition = self.run(condition, ctx)?.ok_or("If condition must return a value")?;

        if !matches!(condition, Value::Boolean(_)) {
            return Err("If condition must return a boolean".to_string())
        }

        if let Value::Boolean(true) = condition {
            self.run(then_block, ctx)
        } else {
            if let Some(else_block) = else_block {
                self.run(else_block, ctx)
            } else {
                Ok(None)
            }
        }
    }

    fn eval_declaration(&self, name: &str, value: &Node, ctx: &Ctx) -> Result<Option<Value>, String> {
        let value = self.run(value, ctx)?.ok_or("Cannot declare a statement without a return value")?;
        ctx.declare(name, value);
        Ok(None)
    }

    pub(super) fn eval_assignment(&self, name: &str, value: &Node, ctx: &Ctx) -> Result<Option<Value>, String> {
        let value = self.run(value, ctx)?.ok_or("Cannot assign a statement without a return value")?;
        ctx.assign(name, value)?;
        Ok(None)
    }
}
