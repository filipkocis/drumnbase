use crate::basics::{Row, Value};

use super::ConditionOperator;

#[derive(Debug)]
pub struct ConditionChain {
    elements: Vec<ChainElement>,
}

#[derive(PartialEq, Debug)]
pub enum ChainElement {
    Condition(Condition),
    And,
    Or,
    Not,
}

#[derive(PartialEq, Debug)]
pub struct Condition {
    column: String,
    operator: ConditionOperator,
    value: Value,
    index: usize, // index of the column in the table
}

impl ConditionChain {
    pub fn new(elements: Vec<ChainElement>) -> Self {
        Self {
            elements
        }
    }

    pub fn check(&self, row: &Row) -> Result<bool, String> {
        let mut result = false;
        let mut chain_operator = &ChainElement::Or;

        if self.elements.len() == 0 {
            // should we return ok or err here ?
            return Ok(true);
        }
        if let Some(first_condition) = self.elements.get(0) {
            if !first_condition.is_condition() && first_condition != &ChainElement::Not {
                return Err(format!("First condition must be a condition, not a chain operator: {:?}", first_condition));
            }
        }

        for element in &self.elements {  
            if element.is_operator() {
                chain_operator = element;
                // continue with the condition, multiple operators should be prevented beforehand, as they will be replaced
                // TODO: add check for mulitiple operators
                continue; 
            }

            let condition = element.get_condition().unwrap();
            let cell = match row.get(condition.index) {
                Some(cell) => cell,
                None => return Err(format!("Column '{}' at '{}' not found", condition.column, condition.index))
            };

            // TODO: add special parsing for functions here (?)

            let check = condition.operator.check(cell, &condition.value); 
            result = chain_operator.evaluate(result, check);
        }

        Ok(result)
    }
}

impl ChainElement {
    fn evaluate(&self, left: bool, right: bool) -> bool {
        match self {
            Self::And => left && right,
            Self::Or => left || right,
            Self::Not => !right,
            _ => false
        }
    }

    pub fn get_condition(&self) -> Option<&Condition> {
        match self {
            Self::Condition(condition) => Some(condition), 
            _ => None
        }
    }

    pub fn is_condition(&self) -> bool {
        match self {
            Self::Condition(_) => true,
            _ => false
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            Self::And | Self::Or | Self::Not => true,
            _ => false
        }
    }
}

impl Condition {
    pub fn new(column: String, operator: ConditionOperator, value: Value, index: usize) -> Self {
        Self {
            column,
            operator,
            value,
            index
        }
    }
}
