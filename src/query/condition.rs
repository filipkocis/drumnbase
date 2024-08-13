use crate::basics::{row::{Row, Value}, column::{ColumnType, Column}};

#[derive(Debug)]
pub struct ConditionChainParsed {
    conditions: Vec<ConditionChainValueParsed>,
}

#[derive(PartialEq, Debug)]
enum ConditionChainValueParsed {
    Condition(ConditionParsed),
    And,
    Or,
    Not,
}

impl ConditionChainValueParsed {
    fn evaluate(&self, left: bool, right: bool) -> bool {
        match self {
            ConditionChainValueParsed::And => left && right,
            ConditionChainValueParsed::Or => left || right,
            ConditionChainValueParsed::Not => !right,
            _ => false
        }
    }
}

#[derive(PartialEq, Debug)]
struct ConditionParsed {
    column: String,
    operator: ConditionOperator,
    value: Value,
    index: usize, // index of the column in the table
}

#[derive(Debug)]
pub struct ConditionChain {
    pub conditions: Vec<ConditionChainValue>,
}

impl ConditionChainValueParsed {
    pub fn get_condition(&self) -> Option<&ConditionParsed> {
        match self {
            ConditionChainValueParsed::Condition(condition) => Some(condition), 
            _ => None
        }
    }

    pub fn is_condition(&self) -> bool {
        match self {
            ConditionChainValueParsed::Condition(_) => true,
            _ => false
        }
    }
}

impl ConditionChainParsed {
    pub fn check(&self, row: &Row) -> Result<bool, String> {
        let mut result = false;
        let mut chain_operator = &ConditionChainValueParsed::Or;

        if self.conditions.len() == 0 {
            // should we return ok or err here ?
            return Ok(true);
        }
        if let Some(first_condition) = self.conditions.get(0) {
            if !first_condition.is_condition() && first_condition != &ConditionChainValueParsed::Not {
                return Err(format!("First condition must be a condition, not a chain operator: {:?}", first_condition));
            }
        }

        for condition in &self.conditions {  
            if !condition.is_condition() {
                chain_operator = condition;
                // continue with the condition, multiple operators should be prevented beforehand, as they will be replaced
                continue; 
            }

            let condition = condition.get_condition().unwrap();
            let cell = row.get(condition.index).ok_or(format!("Column '{}' at '{}' not found", condition.column, condition.index))?;

            let check = condition.operator.check(cell, &condition.value); 
            result = chain_operator.evaluate(result, check);
        }

        Ok(result)
    }

}

impl ConditionChain {
    pub fn get_parsed_value_chain(&self, columns: &Vec<Column>) -> Result<ConditionChainParsed, String> {
        let mut parsed_conditions = Vec::new();

        for condition in &self.conditions {
            let parsed = condition.into_parsed_value(columns)?;
            parsed_conditions.push(parsed)
        }

        let parsed = ConditionChainParsed {
            conditions: parsed_conditions,
        };

        Ok(parsed)
    }
}

#[derive(PartialEq, Debug)]
pub enum ConditionChainValue {
    Condition(Condition),
    And,
    Or,
    Not,
}

impl ConditionChainValue {
    pub fn list() -> Vec<&'static str> {
        vec!["and", "or", "not"]
    }

    pub fn from_str(s: &str) -> Result<ConditionChainValue, String> {
        match s {
            "and" => Ok(ConditionChainValue::And),
            "or" => Ok(ConditionChainValue::Or),
            "not" => Ok(ConditionChainValue::Not),
            _ => Err(format!("Unknown condition chain value: {}", s))
        }
    }

    fn into_parsed_value(&self, columns: &Vec<Column>) -> Result<ConditionChainValueParsed, String> {
        let parsed = match self {
            ConditionChainValue::Condition(condition) => {
                let column_type = &columns
                    .iter()
                    .find(|c| c.name == condition.column)
                    .ok_or(format!("Column '{}' not found", condition.column))?
                    .data_type;

                let parsed = condition.into_parsed_value(column_type, columns)?;
                ConditionChainValueParsed::Condition(parsed)
            },
            ConditionChainValue::And => ConditionChainValueParsed::And,
            ConditionChainValue::Or => ConditionChainValueParsed::Or,
            ConditionChainValue::Not => ConditionChainValueParsed::Not,
        };

        Ok(parsed)
    }
}

#[derive(PartialEq, Debug)]
pub struct Condition {
    pub column: String,
    pub operator: ConditionOperator,
    pub value: String,
}

impl Condition {
    fn into_parsed_value(&self, column_type: &ColumnType, columns: &Vec<Column>) -> Result<ConditionParsed, String> {
        let column = self.column.clone();
        let operator = self.operator.clone();
        let value = column_type.parse(&self.value)?;
        let index = columns.iter().position(|c| c.name == column).ok_or(format!("Column '{}' not found", column))?;

        Ok(ConditionParsed {
            column, operator, value, index
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
    NotBetween,
}

impl ConditionOperator {
    pub fn check(&self, cell: &Value, value: &Value) -> bool {
        match self {
            ConditionOperator::Equal => cell == value,
            ConditionOperator::NotEqual => cell != value,
            ConditionOperator::GreaterThan => cell > value,
            ConditionOperator::LessThan => cell < value,
            ConditionOperator::GreaterThanOrEqual => cell >= value,
            ConditionOperator::LessThanOrEqual => cell <= value,
            ConditionOperator::Like => cell.like(value),
            ConditionOperator::NotLike => !cell.like(value),
            ConditionOperator::In => cell.in_(value),
            ConditionOperator::NotIn => !cell.in_(value),
            ConditionOperator::IsNull => cell.is_null(),
            ConditionOperator::IsNotNull => !cell.is_null(),
            ConditionOperator::Between => cell.between(value),
            ConditionOperator::NotBetween => !cell.between(value),
        }
    }

    pub fn list() -> Vec<&'static str> {
        vec!["=", ">", "<", ">=", "<=", "!=", "like", "in", "not in", "between", "is null", "is not null", "eq", "gt", "lt", "gte", "lte", "ne", "lk", "in", "nin", "btw", "isnull", "notnull"]
    }

    pub fn from_str(s: &str) -> Result<ConditionOperator, String> {
        let op = match s {
            "=" => ConditionOperator::Equal,
            "eq" => ConditionOperator::Equal,

            ">" => ConditionOperator::GreaterThan,
            "gt" => ConditionOperator::GreaterThan,

            "<" => ConditionOperator::LessThan,
            "lt" => ConditionOperator::LessThan,

            ">=" => ConditionOperator::GreaterThanOrEqual,
            "gte" => ConditionOperator::GreaterThanOrEqual,

            "<=" => ConditionOperator::LessThanOrEqual,
            "lte" => ConditionOperator::LessThanOrEqual,

            "!=" => ConditionOperator::NotEqual,
            "ne" => ConditionOperator::NotEqual,

            "like" => ConditionOperator::Like,
            "lk" => ConditionOperator::Like,

            "in" => ConditionOperator::In,
            // "in" => ConditionOperator::In,
            
            "not in" => ConditionOperator::NotIn,
            "nin" => ConditionOperator::NotIn,

            "between" => ConditionOperator::Between,
            "btw" => ConditionOperator::Between,

            "is null" => ConditionOperator::IsNull,
            "isnull" => ConditionOperator::IsNull,

            "is not null" => ConditionOperator::IsNotNull,
            "notnull" => ConditionOperator::IsNotNull,

            _ => return Err(format!("Unknown condition operator: {}", s)) 
        };

        Ok(op)
    }
}
