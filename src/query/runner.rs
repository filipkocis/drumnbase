use crate::database::database::Database;

use super::parser::{SimpleQueryParser, QueryParser};

pub trait QueryRunner {
    fn run_query(&mut self, query: &str) -> Result<(), String>;
}

impl QueryRunner for Database {
    fn run_query(&mut self, query: &str) -> Result<(), String> {
        let query = query.trim();
        if query.is_empty() { return Ok(()); }
        
        let query = SimpleQueryParser::from(query)?.parse()?;
        let result = query.apply_to(self)?;

        // println!("AMOUNT: {:#?}", result.amount);
        // println!("DATA: {:#?}", result.data);

        Ok(())
    }
}
