use crate::{syntax::ast::{Node, Statement}, basics::row::Value};

use super::{Runner, BlockResult};

impl Runner {
    pub(super) fn eval_loop(&self, block: &Box<Node>) -> Result<Value, String> {
        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("Loop block must be a block".to_string())
        };

        loop {
            match self.eval_block(block_nodes)? {
                BlockResult::Return(value) => return Ok(value),
                BlockResult::Break => break,
                BlockResult::Continue |
                BlockResult::End => continue,
            }
        };

        Ok(Value::Null)
    }

    pub(super) fn eval_for(&self, initializer: &Box<Node>, condition: &Box<Node>, action: &Box<Node>, block: &Box<Node>) -> Result<Value, String> {
        if !matches!(**initializer, Node::Statement(Statement::Let { .. })) {
            return Err("For loop initializer must be a let statement".to_string())
        }

        if !matches!(**condition, Node::Expression(_)) {
            return Err("For loop condition must be an expression".to_string())
        }

        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("For loop block must be a block".to_string())
        };

        self.run(initializer)?;

        while let Value::Boolean(true) = self.run(condition)? {
            match self.eval_block(block_nodes)? {
                BlockResult::Return(value) => return Ok(value),
                BlockResult::Break => break,
                BlockResult::Continue |
                BlockResult::End => self.run(action)?,
            };
        }

        Ok(Value::Null)
    }

    pub(super) fn eval_while(&self, condition: &Node, block: &Box<Node>) -> Result<Value, String> {
        if !matches!(condition, Node::Expression(_)) {
            return Err("While condition must be an expression".to_string())
        }

        let block_nodes = match **block {
            Node::Block(ref nodes) => nodes,
            _ => return Err("While block must be a block".to_string())
        };

        while let Value::Boolean(true) = self.run(condition)? {
            match self.eval_block(block_nodes)? {
                BlockResult::Return(value) => return Ok(value),
                BlockResult::Break => break,
                BlockResult::Continue |
                BlockResult::End => continue,
            }
        }

        Ok(Value::Null)
    }
}
