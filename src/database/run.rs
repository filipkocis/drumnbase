use std::sync::{Arc, RwLock};

use crate::{syntax::{runner::Runner, tokenizer::Tokenizer, context::RunnerContext, parser::Parser}, basics::Value};

use super::Database;

pub struct QueryResult {
    pub amount: usize,
    pub data: Value,
}

pub trait Run {
    fn run(database: Arc<RwLock<Database>>, input: String) -> Result<QueryResult, String>;
}

impl Run for Database {
    fn run(database: Arc<RwLock<Database>>, input: String) -> Result<QueryResult, String> {
        let tokens = Tokenizer::new(input).tokenize()?;
        let ast = Parser::new(tokens).parse().or_else(|_| Err("failed to parse".to_string()))?;
        let runner = Runner::new(database);
        let ctx = RunnerContext::new_ctx();

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
                None => Err("no result".to_string()),
            },
            Err(err) => Err(err.to_string())
        }
    }
}
