use crate::{syntax::{ast::{Node, Statement}, context::RunnerContextScope}, basics::row::Value};

use super::{Runner, BlockResult, Ctx};

impl Runner {
    pub(super) fn eval_loop(&self, block: &Box<Node>, ctx: &Ctx) -> Result<Option<Value>, String> {
        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("Loop block must be a block".to_string())
        };

        let inside_loop = self.inside_loop.replace(true); 
        loop {
            match self.eval_block(block_nodes, ctx)? {
                BlockResult::Return(value) => {
                    self.inside_loop.replace(inside_loop);
                    return Ok(Some(value));
                }
                BlockResult::Break => { self.break_loop.replace(false); break; }
                BlockResult::Continue => { self.continue_loop.replace(false); continue; }
                BlockResult::End => continue,
            }
        };
        self.inside_loop.replace(inside_loop);

        Ok(None)
    }

    pub(super) fn eval_for(&self, initializer: &Box<Node>, condition: &Box<Node>, action: &Box<Node>, block: &Box<Node>, ctx: &Ctx) -> Result<Option<Value>, String> {
        match **initializer {
            Node::Statement(Statement::Let { .. }) |
            Node::Statement(Statement::Assignment { .. }) |
            Node::Literal(_) => {},
            _ => return Err("For loop initializer must be a let statement, assignment or literal".to_string())
        };

        if !matches!(**condition, Node::Expression(_)) {
            return Err("For loop condition must be an expression".to_string())
        }

        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("For loop block must be a block".to_string())
        };

        let ctx = &Ctx::scoped(ctx.clone());
        self.run(initializer, ctx)?;

        let inside_loop = self.inside_loop.replace(true); 
        while let Value::Boolean(true) = self.run(condition, ctx)?.ok_or("For loop condition must return a value")? {
            match self.eval_block(block_nodes, ctx)? {
                BlockResult::Return(value) => {
                    self.inside_loop.replace(inside_loop);
                    return Ok(Some(value));
                }
                BlockResult::Break => { self.break_loop.replace(false); break; }
                BlockResult::Continue => { self.continue_loop.replace(false); self.run(action, ctx)?; }
                BlockResult::End => { self.run(action, ctx)?; }
            };
        }
        self.inside_loop.replace(inside_loop);

        Ok(None)
    }

    pub(super) fn eval_while(&self, condition: &Node, block: &Box<Node>, ctx: &Ctx) -> Result<Option<Value>, String> {
        if !matches!(condition, Node::Expression(_)) {
            return Err("While condition must be an expression".to_string())
        }

        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("While block must be a block".to_string())
        };

        let inside_loop = self.inside_loop.replace(true); 
        while let Value::Boolean(true) = self.run(condition, ctx)?.ok_or("While condition must return a value")? {
            match self.eval_block(block_nodes, ctx)? {
                BlockResult::Return(value) => {
                    self.inside_loop.replace(inside_loop);
                    return Ok(Some(value));
                }
                BlockResult::Break => { self.break_loop.replace(false); break; }
                BlockResult::Continue => { self.continue_loop.replace(false); continue; }
                BlockResult::End => continue,
            }
        }
        self.inside_loop.replace(inside_loop);

        Ok(None)
    }
}
