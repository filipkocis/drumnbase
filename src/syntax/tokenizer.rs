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

    


    
   
    /// Parse number token
    fn number(&mut self) -> Result<Token, String> {
        let mut value = String::new();
        let mut is_float = false;
        let mut prev_underscore = false;
        
        while let Some(current) = self.current() {
            match current {
                'a'..='z' | 'A'..='Z' => {
                    return Err(format!("unexpected character {:?} at position {}", current, self.position))
                },
                '0'..='9' => {
                    prev_underscore = false;
                    value.push(current)
                },
                c if c == '.' 
                    && !is_float // only one dot per number 
                    && !prev_underscore  // cannot be preceded by an underscore
                    && value.len() != 0  // cannot be at the beginning
                    && self.match_next('0'..='9') => // must be followed by a digit
                {
                    is_float = true;
                    value.push(current);
                },
                c if c == '_'  && value.len() != 0 => {
                    prev_underscore = true;
                }
                _ => break,
            }

            self.advance();
        }

        if prev_underscore {
            return Err(format!("unexpected underscore at position {:?}", self.position))
        } 
        if value.len() == 0 {
            return Err(format!("unexpected character {:?} at position {}", self.current(), self.position))
        }

        let literal = if is_float {
            Literal::Float(value)
        } else {
            Literal::Int(value)
        };

        Ok(Token {
            kind: TokenKind::Literal(literal),
            position: self.position, 
        })
    }
}
