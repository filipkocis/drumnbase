use std::{sync::{Arc, RwLock}, rc::Rc};

use crate::{syntax::{runner::Runner, tokenizer::Tokenizer, context::{RunnerContext}, parser::Parser}, basics::Value, auth::User, cluster::Cluster};

use super::Database;

pub struct QueryResult {
    pub amount: usize,
    pub data: Value,
}

pub struct RunOptions {
    // additional runner context
    pub cluster_user: Rc<User>,
    pub auth_user: Rc<User>,
    pub cluster: Arc<RwLock<Cluster>>,

    // runner options
    pub is_schema: bool,
}

impl RunOptions {
    pub fn new(cluster_user: Rc<User>, auth_user: Rc<User>, cluster: Arc<RwLock<Cluster>>) -> Self {
        Self {
            cluster_user,
            auth_user,
            cluster,
            is_schema: false,
        }
    }

    pub fn new_rc(cluster_user: Rc<User>, auth_user: Rc<User>, cluster: Arc<RwLock<Cluster>>) -> Rc<Self> {
        Rc::new(Self::new(cluster_user, auth_user, cluster))
    }
}

pub trait Run {
    fn run(database: Arc<RwLock<Database>>, input: String, options: Rc<RunOptions>) -> Result<QueryResult, String>;
}

impl Run for Database {
    fn run(database: Arc<RwLock<Database>>, input: String, options: Rc<RunOptions>) -> Result<QueryResult, String> {
        let tokens = Tokenizer::new(input).tokenize()?;
        let ast = Parser::new(tokens).parse().or_else(|_| Err("failed to parse".to_string()))?;
        let runner = Runner::new(database);
        let ctx = RunnerContext::new_ctx(options);

        match runner.run(&ast, &ctx) {
            Ok(result) => match result {
                Some(result) => match result {
                    Value::Array(array) => {
                        Ok(QueryResult {
                            amount: array.len(),
                            data: Value::Array(array),
                        })
                    },
                    _ => {
                        Ok(QueryResult {
                            amount: 1,
                            data: result,
                        })
                    }
                    // _ => Err("wrong result type".to_string()),
                },
                // None => Err("no result".to_string()),
                None => Ok(QueryResult {
                    amount: 0,
                    data: Value::Null,
                })
            },
            Err(err) => Err(err.to_string())
        }
    }
}
