use std::collections::HashMap;

use crate::{syntax::ast::{Node, Statement}, basics::row::Value};

use super::{Runner, BlockResult};

impl Runner {
    pub(super) fn eval_loop(&self, block: &Box<Node>) -> Result<Option<Value>, String> {
        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("Loop block must be a block".to_string())
        };

        let inside_loop = self.inside_loop.replace(true); 
        loop {
            match self.eval_block(block_nodes)? {
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

    pub(super) fn eval_for(&self, initializer: &Box<Node>, condition: &Box<Node>, action: &Box<Node>, block: &Box<Node>) -> Result<Option<Value>, String> {
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

        let mut saved_scope = HashMap::new();
        if let Node::Statement(Statement::Let { name, .. }) = initializer.as_ref() {
            let scope = self.variables.borrow();
            let value = scope.get(name);
            saved_scope.insert(name.clone(), value.cloned());
        };

        self.run(initializer)?;

        let inside_loop = self.inside_loop.replace(true); 
        while let Value::Boolean(true) = self.run(condition)?.ok_or("For loop condition must return a value")? {
            match self.eval_block(block_nodes)? {
                BlockResult::Return(value) => {
                    self.inside_loop.replace(inside_loop);
                    self.reset_scope(saved_scope);
                    return Ok(Some(value));
                }
                BlockResult::Break => { self.break_loop.replace(false); break; }
                BlockResult::Continue => { self.continue_loop.replace(false); self.run(action)?; }
                BlockResult::End => { self.run(action)?; }
            };
        }
        self.inside_loop.replace(inside_loop);

        self.reset_scope(saved_scope);
        Ok(None)
    }

    pub(super) fn eval_while(&self, condition: &Node, block: &Box<Node>) -> Result<Option<Value>, String> {
        if !matches!(condition, Node::Expression(_)) {
            return Err("While condition must be an expression".to_string())
        }

        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("While block must be a block".to_string())
        };

        let inside_loop = self.inside_loop.replace(true); 
        while let Value::Boolean(true) = self.run(condition)?.ok_or("While condition must return a value")? {
            match self.eval_block(block_nodes)? {
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
