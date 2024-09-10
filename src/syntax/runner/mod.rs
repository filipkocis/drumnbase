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
    Return(Value),
    Break,
    Continue,
}

pub struct Runner {
    pub database: Rc<RefCell<Database>>,
    pub variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl Runner {
    pub fn new(database: Rc<RefCell<Database>>) -> Self {
        Self {
            database,
            variables: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    pub fn run(&self, ast: &Node) -> Result<Value, String> {
        match ast {
            Node::Literal(value) => self.eval_literal(value),
            Node::Block(nodes) => self.eval_pure_block(nodes),
            Node::Statement(statement) => self.eval_statement(statement),
            Node::Expression(expression) => self.eval_expression(expression),
            Node::Query(_) => unimplemented!("query"),
            Node::Value(value) => Ok(value.clone())
        }
    }
}
