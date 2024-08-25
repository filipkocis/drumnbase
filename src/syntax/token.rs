use super::ast;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub index: TokenIndex,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize, line: usize) -> Self {
        Self {
            kind,
            index: TokenIndex { start, end },
            line,
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

impl Keyword {
    pub fn is_literal(&self) -> bool {
        match self {
            Keyword::True | Keyword::False | Keyword::Null => true,
            _ => false
        }
    }
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
    // Asterisk, // not used
    Arrow,
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

impl Operator {
    pub fn to_ast_operator(&self) -> ast::Operator {
        match self {
            // Arithmetic
            Operator::Add => ast::Operator::Add,
            Operator::Subtract => ast::Operator::Sub,
            Operator::Multiply => ast::Operator::Mul,
            Operator::Divide => ast::Operator::Div,
            Operator::Modulus => ast::Operator::Mod,
            Operator::Increment => ast::Operator::Inc,
            Operator::Decrement => ast::Operator::Dec,

            // Assignment
            Operator::Assign => ast::Operator::Assign,
            Operator::AddAssign => ast::Operator::AddAssign,
            Operator::SubtractAssign => ast::Operator::SubAssign,
            Operator::MultiplyAssign => ast::Operator::MulAssign,
            Operator::DivideAssign => ast::Operator::DivAssign,
             
            // Comparison
            Operator::Equal => ast::Operator::Eq,
            Operator::NotEqual => ast::Operator::Ne,
            Operator::GreaterThan => ast::Operator::Gt,
            Operator::GreaterThanOrEqual => ast::Operator::Ge,
            Operator::LessThan => ast::Operator::Lt,
            Operator::LessThanOrEqual => ast::Operator::Le,
             
            // Logical
            Operator::And => ast::Operator::And,
            Operator::Or => ast::Operator::Or,
            Operator::Not => ast::Operator::Not,

            // Bitwise
            Operator::BitwiseAnd => ast::Operator::BitAnd,
            Operator::BitwiseOr => ast::Operator::BitOr,
            Operator::BitwiseNot => ast::Operator::BitNot,
            Operator::BitwiseXor => ast::Operator::BitXor,
            Operator::BitwiseLeftShift => ast::Operator::ShiftLeft,
            Operator::BitwiseRightShift => ast::Operator::ShiftRight,
        }
    }

    pub fn is_assigment(&self) -> bool {
        match self {
            Operator::Assign | Operator::AddAssign | Operator::SubtractAssign | Operator::MultiplyAssign | Operator::DivideAssign => true,
            _ => false
        }
    }

    pub fn is_comparison(&self) -> bool {
        match self {
            Operator::Equal | Operator::NotEqual | Operator::GreaterThan | Operator::GreaterThanOrEqual | Operator::LessThan | Operator::LessThanOrEqual => true,
            _ => false
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        match self {
            Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide | Operator::Modulus => true,
            _ => false
        }
    }

    pub fn is_bitwise(&self) -> bool {
        match self {
            Operator::BitwiseAnd | Operator::BitwiseOr | Operator::BitwiseNot | Operator::BitwiseXor | Operator::BitwiseLeftShift | Operator::BitwiseRightShift => true,
            _ => false
        }
    }

    pub fn is_logical(&self) -> bool {
         match self {
             Operator::And | Operator::Or | Operator::Not => true,
             _ => false
         }
    }

    pub fn is_unary(&self) -> bool {
        match self {
            Operator::Increment | Operator::Decrement | Operator::Not | Operator::BitwiseNot => true,
            _ => false
        }
    }

    pub fn is_binary(&self) -> bool {
        !self.is_unary()
    }

    pub fn returns_boolean(&self) -> bool {
        self.is_comparison() || self.is_logical()
    }

    pub fn returns_same_type(&self) -> bool {
        self.is_arithmetic() || self.is_bitwise()
    }
}
