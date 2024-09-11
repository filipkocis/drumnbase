use crate::{basics::row::Value, syntax::ast::{Node, Statement}};

use super::{Runner, BlockResult};

impl Runner {
    pub(super) fn eval_pure_block(&self, nodes: &Vec<Node>) -> Result<Option<Value>, String> {
        match self.eval_block(nodes)? {
            BlockResult::Return(value) => Ok(Some(value)),
            BlockResult::Break => Err("Break outside of loop".to_string()),
            BlockResult::Continue => Err("Continue outside of loop".to_string()),
            BlockResult::End => Ok(None),
        }
    }

    pub(super) fn eval_block(&self, nodes: &Vec<Node>) -> Result<BlockResult, String> {
        let len = nodes.len() - 1;
        for (i, node) in nodes.iter().enumerate() {
            if let Node::Statement(statement) = node {
                match statement {
                    Statement::Return(value) => {
                        let value = self.run(value)?;
                        match value {
                            Some(value) => return Ok(BlockResult::Return(value)),
                            None => Err("Cannot return a statement without a value".to_string())?
                        }
                    }
                    Statement::Break => return Ok(BlockResult::Break),
                    Statement::Continue => return Ok(BlockResult::Continue),
                    _ => {}
                }
            }

            let value = self.run(node)?;

            if i == len {
                match value {
                    Some(value) => return Ok(BlockResult::Return(value)),
                    None => return Ok(BlockResult::End)
                }
            }

            if matches!(node, Node::Statement(_)) {
                if let Some(value) = value {
                    return Ok(BlockResult::Return(value))
                }
            }
        }

        Ok(BlockResult::End)
    }
}
