use crate::{basics::{Value, Column}, auth::{RlsPolicy, RlsAction, Privilege}};

#[derive(Debug, Clone)]
pub enum Node {
    Block(Vec<Node>),
    Literal(Literal),
    Statement(Statement),
    Expression(Expression),
    Query(Query),
    Value(Value),
    SDL(SDL),
}

#[derive(Debug, Clone)]
pub enum Query {
    // ColumnIndex(usize),
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}

#[derive(Debug, Clone)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Vec<Node>,
    pub where_clause: Option<Box<Node>>,
    pub order: Option<Box<Node>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub table: String,
    pub key_values: Vec<(String, Node)>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub table: String,
    pub key_values: Vec<(String, Node)>,
    pub where_clause: Option<Box<Node>>,
}

#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub table: String,
    pub where_clause: Option<Box<Node>>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Identifier(String),
    Number(Number),
    String(String),
    Boolean(bool),
    Array(Vec<Node>),
    // Object(Vec<(String, Node)>),
    Null,
}

#[derive(Debug, Clone)]
pub enum Number {
    Int(i64),
    UInt(u64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { name: String, value: Box<Node> },
    Expression(Expression),
    Function { name: String, parameters: Vec<(String, Type)>, return_type: Type, block: Box<Node> },
    Let { name: String, value: Box<Node> },
    Return(Box<Node>),
    If { condition: Box<Node>, then_block: Box<Node>, else_block: Option<Box<Node>> },
    While { condition: Box<Node>, block: Box<Node> },
    For { initializer: Box<Node>, condition: Box<Node>, action: Box<Node>, block: Box<Node> },
    Loop { block: Box<Node> },
    Break,
    Continue,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Type {
    Void,
    Int,
    UInt,
    Float,
    String,
    Boolean,
    Pointer(Box<Type>),
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Null,
    Any,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary { left: Box<Node>, operator: Operator, right: Box<Node> },
    Unary { operator: Operator, right: Box<Node> },
    Call { name: String, arguments: Vec<Node> },
    Index { name: String, index: Box<Node> },
    Member { name: String, member: String },
    Literal(Literal),
    Dereference(Box<Node>),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    BitAnd, BitOr, BitXor, BitNot,
    ShiftLeft, ShiftRight,
    
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign, PowAssign,
    Inc, Dec,
}

#[derive(Debug, Clone)]
pub enum SDL {
    Create(CreateSDL),
    Drop(DropSDL), 
    Grant(GrantSDL),
    // Revoke { object: String, from: String, privileges: Vec<Privilege> },
}

#[derive(Debug, Clone)]
pub enum CreateSDL {
    Database { name: String },
    Table { name: String, columns: Vec<Column> },
    RlsPolicy { table: String, policy: Box<RlsPolicy> },
    Role { name: String },
    User { name: String, password: String, is_superuser: bool },
}

#[derive(Debug, Clone)]
// TODO: rework this
pub enum DropSDL {
    Database { name: String },
    Column { name: String, table: String },
    Table { name: String },
    RlsPolicy { name: String },
    Role { name: String },
    User { name: String },
}

#[derive(Debug, Clone)]
pub enum GrantSDL {
    Role { name: String, to: String },
    // TODO: rework this
    Action { object: String, object_name: String, actions: Vec<String>, table: Option<String>, to: String, }
}
