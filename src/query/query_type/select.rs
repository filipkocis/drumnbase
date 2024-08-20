use crate::query::condition::chain::ConditionChain;

use super::Order;

#[derive(Debug)]
pub struct SelectQuery {
    pub columns: Vec<String>,  
    pub extras: Vec<SelectExtra>,
}

#[derive(Debug)]
pub enum SelectExtra {
    Where(ConditionChain),
    Order(Order),
    Limit(usize),
    Offset(usize),
    Exclude(Vec<String>),
}

impl SelectQuery {
    pub fn get_limit(&self) -> Option<usize> {
        for extra in &self.extras {
            if let SelectExtra::Limit(n) = extra {
                return Some(*n)
            }
        }
        None
    }

    pub fn get_offset(&self) -> Option<usize> {
        for extra in &self.extras {
            if let SelectExtra::Offset(n) = extra {
                return Some(*n)
            }
        }
        None
    }

    pub fn get_order(&self) -> Option<&Order> {
        for extra in &self.extras {
            if let SelectExtra::Order(order) = extra {
                return Some(order)
            }
        }
        None
    }

    pub fn get_exclude(&self) -> Option<&Vec<String>> {
        for extra in &self.extras {
            if let SelectExtra::Exclude(cols) = extra {
                return Some(cols)
            }
        }
        None
    }

    pub fn get_where(&self) -> Option<&ConditionChain> {
        for extra in &self.extras {
            if let SelectExtra::Where(chain) = extra {
                return Some(chain)
            }
        }
        None
    }
}

impl SelectExtra {
    pub fn list() -> Vec<&'static str> {
        vec!["where", "order", "limit", "offset", "exclude"]
    }

    pub fn unwrap_chain(self) -> Result<ConditionChain, String> {
        match self {
            Self::Where(chain) => Ok(chain),
            _ => Err(format!("Expected Where, got {:?}", self))
        }
    }

    pub fn unwrap_limit(self) -> Result<usize, String> {
        match self {
            Self::Limit(n) => Ok(n),
            _ => Err(format!("Expected Limit, got {:?}", self))
        }
    }
}
