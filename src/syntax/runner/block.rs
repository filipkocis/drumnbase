use crate::{basics::row::Value, syntax::ast::{Node, Statement}};

use super::{Runner, BlockResult};

impl Runner {
    pub(super) fn eval_pure_block(&self, nodes: &Vec<Node>) -> Result<Value, String> {
        match self.eval_block(nodes)? {
            BlockResult::Return(value) => Ok(value),
            BlockResult::Break => Err("Break outside of loop".to_string()),
            BlockResult::Continue => Err("Continue outside of loop".to_string())
        }
    }

    pub(super) fn eval_block(&self, nodes: &Vec<Node>) -> Result<BlockResult, String> {
        for node in nodes {
            if let Node::Statement(statement) = node {
                match statement {
                    Statement::Return(value) =>
                        return Ok(BlockResult::Return(self.run(value)?)), 
                    Statement::Break => return Ok(BlockResult::Break),
                    Statement::Continue => return Ok(BlockResult::Continue),
                    _ => {}
                }
            }

            let value = self.run(node)?;
            if let Value::Null = value {
                continue;
            } else {
                return Ok(BlockResult::Return(value))
            }
        }

        Ok(BlockResult::Return(Value::Null))
    }
}
