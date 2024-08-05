#[derive(Debug)]
pub struct ConditionChain {
    pub conditions: Vec<ConditionChainValue>,
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
}

#[derive(PartialEq, Debug)]
pub struct Condition {
    pub column: String,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(PartialEq, Debug)]
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
