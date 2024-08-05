use super::query::{Query, QueryType, SelectExtra, SelectQuery};
use super::condition::{ConditionChain, ConditionOperator, ConditionChainValue, Condition};

pub trait QueryParser {
    fn parse(&mut self) -> Result<Query, String>;
}

pub struct SimpleQueryParser {
    query: String,
    parts: Vec<String>,
    position: usize,
}

impl SimpleQueryParser {
    fn new(query: &str, parts: Vec<String>) -> Self {
        Self {
            query: query.to_string(),
            parts,
            position: 0,
        }
    }

    pub fn from(query: &str) -> Result<Self, String> {
        let query = query.trim();  
        if query.is_empty() { return Err("Empty query".to_string()) }

        let parts: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();
        if parts.len() == 0 { return Err("Invalid query".to_string()) }    

        Ok(Self::new(query, parts))
    }

    fn next(&mut self) -> Option<&str> {
        if self.position >= self.parts.len() { return None; }
        
        let part = &self.parts[self.position];
        self.position += 1;
        Some(part)
    }

    fn peek(&self) -> Option<&str> {
        if self.position >= self.parts.len() { return None; }
        self.get(self.position)
    }

    fn get(&self, position: usize) -> Option<&str> {
        if position >= self.parts.len() { return None; }
        Some(&self.parts[position]) 
    }

    fn expect_next(&mut self, expected: &str) -> Result<(), String> {
        match self.next() {
            Some(part) if part == expected => Ok(()),
            Some(part) => Err(format!("Expected '{}', found '{}'", expected, part)),
            None => Err(format!("Expected '{}', found end of query", expected))
        }
    }

    fn expect_peek(&mut self, expected: &str) -> Result<(), String> {
        match self.peek() {
            Some(part) if part == expected => Ok(()),
            Some(part) => Err(format!("Expected '{}', found '{}'", expected, part)),
            None => Err(format!("Expected '{}', found end of query", expected))
        }
    }

    fn expect_any_next(&mut self, expected: &[&str]) -> Result<&str, String> {
        match self.next() {
            Some(part) if expected.contains(&part) => Ok(part),
            Some(part) => Err(format!("Expected any of '{:?}', found '{}'", expected, part)),
            None => Err(format!("Expected any of '{:?}', found end of query", expected))
        }
    }
    
    fn expect_any_peek(&mut self, expected: &[&str]) -> Result<&str, String> {
        match self.peek() {
            Some(part) if expected.contains(&part) => Ok(part),
            Some(part) => Err(format!("Expected any of '{:?}', found '{}'", expected, part)),
            None => Err(format!("Expected any of '{:?}', found end of query", expected))
        }
    }

    



    fn parse_insert(&mut self) -> Result<QueryType, String> {
        self.expect_next("insert")?;
        // let columns = self.parse_columns()?;

        todo!()
    }



    fn parse_update(&mut self) -> Result<QueryType, String> {
        self.expect_next("update")?;
        // let columns = self.parse_columns()?;

        todo!()
    }



    fn parse_delete(&mut self) -> Result<QueryType, String> {
        self.expect_next("delete")?;
        // let columns = self.parse_columns()?;

        todo!()
    }
}

impl QueryParser for SimpleQueryParser {
    fn parse(&mut self) -> Result<Query, String> {
        if self.parts.len() == 0 { return Err("Empty query".to_string()) }
        if self.peek().unwrap() == "query" { self.next(); }

        let table_name = self.next().ok_or("Expected table name")?.to_string();
        let mut query = Query::new(&self.query, &table_name);
         
        let command = self.expect_any_peek(&["select", "insert", "update", "delete"])?;
        let specific_query = match command {
            "select" => self.parse_select(),
            "insert" => self.parse_insert(),
            "update" => self.parse_update(),
            "delete" => self.parse_delete(),
            _ => Err(format!("Invalid query command '{}'", command)) 
        }?;

        query.set_specific(specific_query);

        Ok(query)
    }
}

/// Implementation for SELECT queries
impl SimpleQueryParser {
    fn parse_select(&mut self) -> Result<QueryType, String> {
        self.expect_next("select")?;
        let columns = self.parse_columns()?;
        let extras = self.parse_select_extras()?; 

        Ok(QueryType::Select(SelectQuery { columns, extras }))
    }

    fn parse_columns(&mut self) -> Result<Vec<String>, String> {
        let mut columns = Vec::new();
        
        loop {
            if let None = self.peek() { break; }
            if let Ok(v) = self.expect_any_peek(&SelectExtra::list()) {
                if columns.len() == 0 { return Err(format!("Expected column name, found '{}'", v)) }
                break;
            };

            let column = self.next().unwrap().to_string();
            columns.push(column)
        };

        Ok(columns)
    }
    
    fn parse_select_extras(&mut self) -> Result<Vec<SelectExtra>, String> {
        let mut extras = Vec::new();

        loop {
            if let None = self.peek() { break; }
            if let Ok(v) = self.expect_any_peek(&SelectExtra::list()) {
                match v {
                    "where" => extras.push(self.parse_where()?),
                    "order" => extras.push(self.parse_order()?),
                    "limit" => extras.push(self.parse_limit()?),
                    "offset" => extras.push(self.parse_offset()?),
                    "exclude" => extras.push(self.parse_exclude()?),
                    _ => return Err(format!("Invalid select extra '{}'", v))
                };
            } else {
                return Err(format!("Expected select extra, found '{}'", self.peek().unwrap()))
            }
        }

        Ok(extras)
    }

    fn parse_where(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("where")?; 
        let mut conditions = Vec::new();

        loop {
            if let None = self.peek() { break; }

            if let Ok(v) = self.expect_any_peek(&ConditionChainValue::list()) {
                let chain = ConditionChainValue::from_str(v)?;
                if chain != ConditionChainValue::Not && conditions.len() == 0 {
                    return Err(format!("Condition chain operator '{}' cannot be used at the beggining", v)) 
                }
                self.next();
                conditions.push(chain);
            } else { if conditions.len() != 0 { break; } }
            
            let column = self.next().unwrap().to_string();
            let operator = self.expect_any_next(&ConditionOperator::list())?;
            let operator = ConditionOperator::from_str(operator)?;
            let value = self.next().ok_or("Condition value expected")?.to_string();
           
            let condition = ConditionChainValue::Condition(Condition { column, operator, value });
            conditions.push(condition);
        } 

        let chain = ConditionChain { conditions };
        Ok(SelectExtra::Where(chain))
    }

    fn parse_order(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("order")?; 
        todo!()
    }

    fn parse_limit(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("limit")?; 
        todo!()
    }

    fn parse_offset(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("where")?; 
        todo!()
    }

    fn parse_exclude(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("exclude")?; 
        todo!()
    }
}
