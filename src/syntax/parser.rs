use std::fmt::Debug;

use crate::syntax::token::TokenKind;

use super::{token::{Token, Keyword, Symbol, Literal}, ast::{Node, Statement, Number, self}};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) {
        self.current += 1;
    }
}
