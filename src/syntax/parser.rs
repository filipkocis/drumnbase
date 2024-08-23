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

    /// Expects the current token to be of the specified kind, consumes and returns it.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, String> {
        self.advance();
        let token = self.previous();

        match token {
            Some(token) if token.kind == kind => Ok(token),
            _ => Err(self.expected(kind))
        }
    }

    fn expected(&self, expected: impl Debug) -> String {
        if let Some(token) = self.current() {
            format!("Expected {:?} but found {:?}", expected, token.kind)
        } else {
            format!("Expected {:?} but found None", expected)
        }
    }

    /// Returns the current token, or an error if there is none.
    fn current_token(&self) -> Result<&Token, &str> {
        self.current().ok_or("No token found")
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
