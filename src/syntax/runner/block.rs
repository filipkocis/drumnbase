use crate::{syntax::{ast::{Node, Statement}, context::RunnerContextScope}};

use super::{Runner, BlockResult, Ctx, RunnerResult};

impl Runner {
    pub(super) fn eval_pure_block(&self, nodes: &Vec<Node>, ctx: &Ctx) -> RunnerResult {
        match self.eval_block(nodes, ctx)? {
            BlockResult::Return(value) => Ok(Some(value)),
            BlockResult::Break => {
                if *self.inside_loop.borrow() {
                    Ok(None)
                } else {
                    Err("Break outside of loop".to_string())
                }
            },
            BlockResult::Continue => {
                if *self.inside_loop.borrow() {
                    Ok(None)
                } else {
                    Err("Continue outside of loop".to_string())
                }
            },
            BlockResult::End => Ok(None),
        }
    }

    pub(super) fn eval_block(&self, nodes: &Vec<Node>, ctx: &Ctx) -> Result<BlockResult, String> {
        let len = nodes.len() - 1;
        let ctx = &Ctx::scoped(ctx.clone());

        for (i, node) in nodes.iter().enumerate() {
            if let Node::Statement(statement) = node {
                match statement {
                    Statement::Return(value) => {
                        let value = match self.run(value, ctx) {
                            Ok(value) => value,
                            Err(e) => return Err(e)
                        };
                        match value {
                            Some(value) => return Ok(BlockResult::Return(value)),
                            None => return Err("Cannot return a statement without a value".to_string())
                        }
                    }
                    Statement::Break => {
                        if !*self.inside_loop.borrow() {
                            Err("Break outside of loop".to_string())?
                        }
                        self.break_loop.replace(true);
                        return Ok(BlockResult::Break);
                    },
                    Statement::Continue => {
                        if !*self.inside_loop.borrow() {
                            Err("Continue outside of loop".to_string())?
                        }
                        self.continue_loop.replace(true);
                        return Ok(BlockResult::Continue);
                    },
                    _ => {}
                }
            }

            let value = match self.run(node, ctx) {
                Ok(value) => value,
                Err(e) => return Err(e)
            };

            if *self.break_loop.borrow() { return Ok(BlockResult::Break); }
            if *self.continue_loop.borrow() { return Ok(BlockResult::Continue); }

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
