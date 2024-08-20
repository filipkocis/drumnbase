use crate::basics::row::Value;

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
