use super::{Order, UpdateQuery, Query, QueryType, SelectExtra, SelectQuery, InsertQuery, KeyVal, DeleteQuery};
use super::condition::{
    ConditionOperator,
    chain::{ConditionChain, ChainElement, Condition}
};

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

/// Implementation for DELETE queries
impl SimpleQueryParser {
    fn parse_delete(&mut self) -> Result<QueryType, String> {
        self.expect_next("delete")?;
        let condition_chain = self.parse_where()?.unwrap_chain()?;

        let limit = match self.peek() {
            Some(v) if v == "limit" => { 
                let limit_value = self.parse_limit()?.unwrap_limit()?;
                Some(limit_value)
            },
            Some(v) => return Err(format!("Unexpected token in delete query '{}'", v)),
            None => None
        };

        Ok(QueryType::Delete(DeleteQuery { condition_chain, limit }))
    }

}

/// Implementation for UPDATE queries
impl SimpleQueryParser {
    fn parse_update(&mut self) -> Result<QueryType, String> {
        self.expect_next("update")?;
        let key_vals = self.parse_key_vals()?;
        let condition_chain = self.parse_where()?.unwrap_chain()?;

        if let Some(v) = self.peek() { return Err(format!("Unexpected token in update query '{}'", v)) }

        Ok(QueryType::Update(UpdateQuery { key_vals, condition_chain }))
    }
}

/// Implementation for INSERT queries
impl SimpleQueryParser {
    fn parse_insert(&mut self) -> Result<QueryType, String> {
        self.expect_next("insert")?;
        let key_vals = self.parse_key_vals()?;

        if let Some(v) = self.peek() { return Err(format!("Unexpected token in insert query '{}'", v)) }
        
        Ok(QueryType::Insert(InsertQuery { key_vals }))
    }

    fn parse_key_vals(&mut self) ->Result<Vec<KeyVal>, String> {
        let mut key_vals = Vec::new();

        loop {
            if let None = self.peek() { break; }
            if let Ok(_) = self.expect_any_peek(&SelectExtra::list()) { break; }
            let (column_name, value) = self.parse_key_val()?;
            key_vals.push(KeyVal::from(column_name, value))
        }

        if key_vals.len() == 0 { return Err("At least one keyval expected".to_string()) }

        Ok(key_vals)
    }

    fn parse_key_val(&mut self) -> Result<(String, String), String> {
        let key_val = self.next().ok_or("Column name and value keyval expected")?;
        let parts: Vec<&str> = key_val.split(":").collect();
        if parts.len() != 2 || parts[1].len() == 0 { return Err(format!("Invalid keyval format, expected 'key:val' got '{}'", key_val)) } 

        let key = parts[0].to_string();
        let val = parts[1].to_string();

        Ok((key, val))
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

        let mut where_found = false;
        let mut order_found = false;
        let mut limit_found = false;
        let mut offset_found = false;
        let mut exclude_found = false;

        let dupe_check = |found: &mut bool, name: &str| -> Result<(), String> {
            if *found { return Err(format!("Duplicate select extra '{}'", name)) }
            *found = true;
            Ok(())
        };

        loop {
            if let None = self.peek() { break; }
            if let Ok(v) = self.expect_any_peek(&SelectExtra::list()) {
                match v {
                    "where" => { dupe_check(&mut where_found, v)?; extras.push(self.parse_where()?); },
                    "order" => { dupe_check(&mut order_found, v)?; extras.push(self.parse_order()?); },
                    "limit" => { dupe_check(&mut limit_found, v)?; extras.push(self.parse_limit()?); },
                    "offset" => { dupe_check(&mut offset_found, v)?; extras.push(self.parse_offset()?); },
                    "exclude" => { dupe_check(&mut exclude_found, v)?; extras.push(self.parse_exclude()?); },
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
        let mut elements = Vec::new();

        loop {
            if let None = self.peek() { break; }

            if let Ok(v) = self.expect_any_peek(&ChainElement::list()) {
                let chain_operator = ChainElement::from_str(v)?;
                if chain_operator != ChainElement::Not && elements.len() == 0 {
                    return Err(format!("Condition chain operator '{}' cannot be used at the beggining", v)) 
                }
                self.next();
                elements.push(chain_operator);
            } else { if elements.len() != 0 { break; } }
             
            let column = self.next().unwrap().to_string();
            let operator = self.expect_any_next(&ConditionOperator::list())?;
            let operator = ConditionOperator::from_str(operator)?;
            let value = self.next().ok_or("Condition value expected")?.to_string();
           
            let condition = ChainElement::Condition(Condition { column, operator, value });
            elements.push(condition);
        } 

        if elements.len() == 0 { return Err("At least one condition expected after 'where'".to_string()) }

        let chain = ConditionChain::new(elements);
        Ok(SelectExtra::Where(chain))
    }

    fn parse_order(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("order")?; 
        let column = self.next().ok_or("Order column expected")?.to_string();
        let order = match self.expect_any_next(&["asc", "desc"])? {
            "asc" => Order::Ascending(column),
            "desc" => Order::Descending(column),
            v => return Err(format!("Invalid order direction '{}'", v))
        };

        Ok(SelectExtra::Order(order))
    }

    fn parse_limit(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("limit")?; 
        let limit = self.next().ok_or("Limit value expected")?;
        let limit = limit.parse::<usize>().map_err(|_| format!("Invalid limit value '{}'", limit))?;

        Ok(SelectExtra::Limit(limit))
    }

    fn parse_offset(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("offset")?; 
        let offset = self.next().ok_or("Offset value expected")?;
        let offset = offset.parse::<usize>().map_err(|_| format!("Invalid offset value '{}'", offset))?;

        Ok(SelectExtra::Offset(offset))
    }

    fn parse_exclude(&mut self) -> Result<SelectExtra, String> {
        self.expect_next("exclude")?; 
        let mut columns = Vec::new();

        loop {
            if let None = self.peek() { break; }
            if let Ok(v) = self.expect_any_peek(&SelectExtra::list()) {
                if columns.len() == 0 { return Err(format!("Column name expected after 'exclude', got '{}'", v)) }
                break; 
            }
            columns.push(self.next().unwrap().to_string());
        };

        if columns.len() == 0 { return Err("At least one column name expected after 'exclude'".to_string()) }
        Ok(SelectExtra::Exclude(columns))
    }
}
