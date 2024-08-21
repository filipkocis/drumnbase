use std::ops::RangeBounds;

use super::token::{Token, TokenKind, Symbol, Operator, Literal, Keyword};

pub struct Tokenizer {
    input: String,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Tokenizer {
        Tokenizer {
            input,
            position: 0,
        }
    }

    /// Tokenize input and return a list of tokens or an error
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            let token = self.token()?;
            if token.kind == TokenKind::EOF { break; }
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Get current character
    fn current(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Get next character
    fn next(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }
 
    /// Advance position
    fn advance(&mut self) {
        self.position += 1;
    }

    /// Check if end of file is reached
    fn eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Match next character with expected range
    fn match_next(&self, expected: impl RangeBounds<char>) -> bool {
        let next = self.next();
        next.is_some() && expected.contains(&next.unwrap())
    }

    /// Match next character with expected character
    fn is_next(&self, expected: char) -> bool {
        let next = self.next();
        next.is_some() && next.unwrap() == expected
    }

    /// Parse the next token
    fn token(&mut self) -> Result<Token, String> {
        if self.eof() {
            return Ok(Token {
                kind: TokenKind::EOF,
                position: self.position,
            })
        }

        let current = self.current().unwrap();
        let token = match current {
            ' ' | '\t' | '\n' => {
                self.advance();
                return self.token();
            },
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword()?,
            '"' | '\'' => self.string()?,
            _ => self.symbol_or_operator()?,
        };

        Ok(token)
    }
}
