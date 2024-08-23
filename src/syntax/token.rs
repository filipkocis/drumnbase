#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub index: TokenIndex,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            index: TokenIndex { start, end },
        }
    }

    pub fn start(&self) -> usize {
        self.index.start
    }
    
    pub fn end(&self) -> usize {
        self.index.end
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenIndex {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Literal(Literal),
    Identifier(String),
    Operator(Operator),
    Keyword(Keyword),
    Symbol(Symbol),
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(String),
    Float(String),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    If,
    Else,
    While,
    For,
    Function,
    Return,
    Break,
    Continue,
    Let,
    Const,
    True,
    False,
    Null,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    Comma,
    Colon,
    Semicolon,
    Period,
    DoublePeriod,
    Ellipsis,
    QuestionMark,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Asterisk,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add, 
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Increment,
    Decrement,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    And,
    Or,
    Not,
    BitwiseAnd,
    BitwiseOr,
    BitwiseNot,
    BitwiseXor,
    BitwiseLeftShift,
    BitwiseRightShift,
}
