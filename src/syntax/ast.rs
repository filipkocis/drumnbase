#[derive(Debug)]
pub enum Node {
    Block(Vec<Node>),
    Literal(Literal),
    Statement(Statement),
}

#[derive(Debug)]
pub enum Literal {
    Identifier(String),
    Number(Number),
    String(String),
    Boolean(bool),
    Array(Vec<Node>),
    Null,
}

#[derive(Debug)]
pub enum Number {
    Int(i64),
    UInt(u64),
    Float(f64),
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Function { name: String, parameters: Vec<String>, return_type: Type, block: Box<Node> },
    Let { name: String, value: Box<Node> },
    Return(Box<Node>),
    If { condition: Box<Node>, then_block: Box<Node>, else_block: Option<Box<Node>> },
    While { condition: Box<Node>, block: Box<Node> },
    For { initializer: Box<Node>, condition: Box<Node>, action: Box<Node>, block: Box<Node> },
    Loop { block: Box<Node> },
    Break,
    Continue,
}

#[derive(Debug)]
pub enum Type {
    Void,
    Int,
    UInt,
    Float,
    String,
    Boolean,
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Function(Vec<Type>, Box<Type>),
    Null,
}

#[derive(Debug)]
pub enum Expression {
    Binary { left: Box<Node>, operator: Operator, right: Box<Node> },
    Unary { operator: Operator, right: Box<Node> },
    Call { name: String, arguments: Vec<Node> },
    Index { name: String, index: Box<Node> },
    Member { name: String, member: String },
    Literal(Literal),
    Dereference(Box<Node>),
}

#[derive(Debug)]
pub enum Operator {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    BitAnd, BitOr, BitXor, BitNot,
    ShiftLeft, ShiftRight,
}
