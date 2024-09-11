use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::{basics::row::Value, database::database::Database};

use super::ast::{Node};

mod block;
mod literal;
mod statement;
mod r#loop;
mod expression;
mod math;
mod operator;
mod function;
mod r#type;

enum BlockResult {
    /// Either a return statement or the last auto-returned value
    Return(Value),
    /// A break statement (valid in loops only)
    Break,
    /// A continue statemnt (valid in loops only)
    Continue,
    /// Block reached the end without a return statement
    End,
}

pub struct Runner {
    pub database: Rc<RefCell<Database>>,
    pub variables: Rc<RefCell<HashMap<String, Value>>>,

    inside_loop: RefCell<bool>,
    break_loop: RefCell<bool>,
    continue_loop: RefCell<bool>,
}

impl Runner {
    pub fn new(database: Rc<RefCell<Database>>) -> Self {
        Self {
            database,
            variables: Rc::new(RefCell::new(HashMap::new())),

            inside_loop: RefCell::new(false),
            break_loop: RefCell::new(false),
            continue_loop: RefCell::new(false),
        }
    }

    pub fn run(&self, ast: &Node) -> Result<Option<Value>, String> {
        match ast {
            Node::Literal(value) => self.eval_literal(value),
            Node::Block(nodes) => self.eval_pure_block(nodes),
            Node::Statement(statement) => self.eval_statement(statement),
            Node::Expression(expression) => self.eval_expression(expression),
            Node::Query(_) => unimplemented!("query"),
            Node::Value(value) => Ok(Some(value.clone()))
        }
    }
}
