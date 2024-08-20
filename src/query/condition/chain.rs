use crate::basics::{column::{ColumnType, Column}};

use super::{ConditionOperator, parsed};

#[derive(Debug)]
pub struct ConditionChain {
    pub elements: Vec<ChainElement>,
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
    pub column: String,
    pub operator: ConditionOperator,
    pub value: String,
}

impl ConditionChain {
    pub fn new(elements: Vec<ChainElement>) -> Self {
        Self {
            elements
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn get_parsed_value_chain(&self, columns: &Vec<Column>) -> Result<parsed::ConditionChain, String> {
        let mut parsed_elements = Vec::new();

        for element in &self.elements {
            let parsed = element.into_parsed(columns)?;
            parsed_elements.push(parsed)
        }

        let parsed = parsed::ConditionChain::new(parsed_elements);
        Ok(parsed)
    }
}

impl ChainElement {
    pub fn list() -> Vec<&'static str> {
        vec!["and", "or", "not"]
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            "not" => Ok(Self::Not),
            _ => Err(format!("Unknown condition chain value: {}", s))
        }
    }

    fn into_parsed(&self, columns: &Vec<Column>) -> Result<parsed::ChainElement, String> {
        let parsed = match self {
            Self::Condition(condition) => {
                let column_type = &columns
                    .iter()
                    .find(|c| c.name == condition.column)
                    .ok_or(format!("Column '{}' not found", condition.column))?
                    .data_type;

                let parsed = condition.into_parsed(column_type, columns)?;
                parsed::ChainElement::Condition(parsed)
            },
            Self::And => parsed::ChainElement::And,
            Self::Or => parsed::ChainElement::Or,
            Self::Not => parsed::ChainElement::Not,
        };

        Ok(parsed)
    }
}

impl Condition {
    fn into_parsed(&self, column_type: &ColumnType, columns: &Vec<Column>) -> Result<parsed::Condition, String> {
        let column = self.column.clone();
        let operator = self.operator.clone();
        let value = column_type.parse(&self.value)?;
        let index = columns.iter().position(|c| c.name == column).ok_or(format!("Column '{}' not found", column))?;

        Ok(parsed::Condition::new(column, operator, value, index))
    }
}
