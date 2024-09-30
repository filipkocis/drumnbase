use std::{cell::RefCell, sync::{Arc, RwLock}};

use crate::{basics::Value, database::Database, lock::UnsafeRwLock};

use super::{ast::{Node}, context::Ctx};

mod block;
mod literal;
mod statement;
mod r#loop;
mod expression;
mod math;
mod operator;
mod function;
mod r#type;
mod query;

type RunnerResult = Result<Option<Value>, String>;

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
    pub database: UnsafeRwLock<Database>,
    // pub variables: Rc<RefCell<HashMap<String, Value>>>,
    inside_loop: RefCell<bool>,
    break_loop: RefCell<bool>,
    continue_loop: RefCell<bool>,
}

impl Runner {
    pub fn new(database: Arc<RwLock<Database>>) -> Self {
        Self {
            database: UnsafeRwLock::new(database),
            // variables: Rc::new(RefCell::new(HashMap::new())),
            inside_loop: RefCell::new(false),
            break_loop: RefCell::new(false),
            continue_loop: RefCell::new(false),
        }
    }

    pub fn run(&self, ast: &Node, ctx: &Ctx) -> RunnerResult {
        match ast {
            Node::Literal(value) => self.eval_literal(value, ctx),
            Node::Block(nodes) => self.eval_pure_block(nodes, ctx),
            Node::Statement(statement) => self.eval_statement(statement, ctx),
            Node::Expression(expression) => self.eval_expression(expression, ctx),
            Node::Query(query) => self.eval_query(query, ctx),
            Node::Value(value) => Ok(Some(value.clone()))
        }
    }
}
