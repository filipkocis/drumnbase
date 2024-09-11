use std::collections::HashMap;

use crate::{basics::row::Value, syntax::ast::{Node, Statement}};

use super::{Runner, BlockResult};

impl Runner {
    pub(super) fn eval_pure_block(&self, nodes: &Vec<Node>) -> Result<Option<Value>, String> {
        match self.eval_block(nodes)? {
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

    pub(super) fn eval_block(&self, nodes: &Vec<Node>) -> Result<BlockResult, String> {
        let len = nodes.len() - 1;
        let mut saved_scope = HashMap::new();

        for (i, node) in nodes.iter().enumerate() {
            if let Node::Statement(statement) = node {
                match statement {
                    Statement::Return(value) => {
                        let value = match self.run(value) {
                            Ok(value) => value,
                            Err(e) => { self.reset_scope(saved_scope); return Err(e) }
                        };
                        self.reset_scope(saved_scope);
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
                        self.reset_scope(saved_scope);
                        return Ok(BlockResult::Break);
                    },
                    Statement::Continue => {
                        if !*self.inside_loop.borrow() {
                            Err("Continue outside of loop".to_string())?
                        }
                        self.continue_loop.replace(true);
                        self.reset_scope(saved_scope);
                        return Ok(BlockResult::Continue);
                    },
                    Statement::Let { name, .. } => {
                        let scope = self.variables.borrow();
                        let saved_value = scope.get(name);
                        saved_scope.insert(name.clone(), saved_value.cloned()); 
                    }
                    _ => {}
                }
            }

            let value = match self.run(node) {
                Ok(value) => value,
                Err(e) => { self.reset_scope(saved_scope); return Err(e) }
            };

            if *self.break_loop.borrow(){ 
                self.reset_scope(saved_scope); 
                return Ok(BlockResult::Break); 
            }
            if *self.continue_loop.borrow() { 
                self.reset_scope(saved_scope); 
                return Ok(BlockResult::Continue); 
            }

            if i == len {
                self.reset_scope(saved_scope); 
                match value {
                    Some(value) => return Ok(BlockResult::Return(value)),
                    None => return Ok(BlockResult::End)
                }
            }

            if matches!(node, Node::Statement(_)) {
                if let Some(value) = value {
                    self.reset_scope(saved_scope); 
                    return Ok(BlockResult::Return(value))
                }
            }
        }

        self.reset_scope(saved_scope); 
        Ok(BlockResult::End)
    }
}
