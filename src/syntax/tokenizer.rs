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

    pub fn error(&self, message: &str, expected: &str) -> String {
        format!("{} at position {:?}, expected {:?}, got {:?}", message, self.position, expected, self.current())
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

    /// Parse symbol or operator token
    fn symbol_or_operator(&mut self) -> Result<Token, String> {
        let current = self.current().expect("unexpected end of file, expected symbol or operator");

        let symbol = match current {
            ',' => Some(Symbol::Comma),
            ':' => Some(Symbol::Colon),
            ';' => Some(Symbol::Semicolon),
            '.' => {
                let mut count = 1;

                while self.is_next('.') {
                    count += 1;
                    self.advance();
                }

                match count {
                    1 => Some(Symbol::Period),
                    2 => Some(Symbol::DoublePeriod),
                    3 => Some(Symbol::Ellipsis),
                    _ => Err(self.error("unexpected character", "max 3 periods"))?
                }
            }
            '?' => Some(Symbol::QuestionMark),
            '(' => Some(Symbol::LeftParenthesis),
            ')' => Some(Symbol::RightParenthesis),
            '{' => Some(Symbol::LeftBracket),
            '}' => Some(Symbol::RightBracket),
            '[' => Some(Symbol::LeftBrace),
            ']' => Some(Symbol::RightBrace),
            _ => None,
        };

        if let Some(symbol) = symbol {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Symbol(symbol),
                position: self.position, 
            })
        }

        let adv_ret = |s: &mut Tokenizer, op| {
            s.advance();
            op
        };

        let operator = match current {
            '%' => Some(Operator::Modulus), 
            '~' => Some(Operator::BitwiseNot), 
            '^' => Some(Operator::BitwiseXor), 

            c => match (c, self.next()) {
                ('&', Some('&')) => adv_ret(self, Some(Operator::And)),
                ('&', _) => Some(Operator::BitwiseAnd),
                ('|', Some('|')) => adv_ret(self, Some(Operator::Or)),
                ('|', _) => Some(Operator::BitwiseOr),

                ('<', Some('<')) => adv_ret(self, Some(Operator::BitwiseLeftShift)),
                ('<', Some('=')) => adv_ret(self, Some(Operator::LessThanOrEqual)),
                ('<', _) => Some(Operator::LessThan),

                ('>', Some('>')) => adv_ret(self, Some(Operator::BitwiseRightShift)),
                ('>', Some('=')) => adv_ret(self, Some(Operator::GreaterThanOrEqual)),
                ('>', _) => Some(Operator::GreaterThan),

                ('!', Some('=')) => adv_ret(self, Some(Operator::NotEqual)),
                ('!', _) => Some(Operator::Not),

                ('=', Some('=')) => adv_ret(self, Some(Operator::Equal)),
                ('=', _) => Some(Operator::Assign),

                ('+', Some('+')) => adv_ret(self, Some(Operator::Increment)),
                ('+', Some('=')) => adv_ret(self, Some(Operator::AddAssign)),
                ('+', _) => Some(Operator::Add),

                ('-', Some('-')) => adv_ret(self, Some(Operator::Decrement)),
                ('-', Some('=')) => adv_ret(self, Some(Operator::SubtractAssign)),
                ('-', _) => Some(Operator::Subtract),

                ('*', Some('=')) => adv_ret(self, Some(Operator::MultiplyAssign)),
                ('*', _) => Some(Operator::Multiply),

                ('/', Some('=')) => adv_ret(self, Some(Operator::DivideAssign)),
                ('/', _) => Some(Operator::Divide),

                _ => None,
            }
        };

        if let Some(operator) = operator {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Operator(operator),
                position: self.position,
            })
        }

        Err(self.error("unexpected character", "symbol or operator"))
    }

    /// Parse string token
    fn string(&mut self) -> Result<Token, String> {
        let mut value = String::new();
        let mut escaped = false;
        let mut closed = false;
        
        if !matches!(self.current(), Some('"') | Some('\'')) {
            Err(self.error("unexpected character", "quotes"))?
        }
        let quote = self.current().unwrap();
        self.advance();

        while let Some(current) = self.current() {
            match current {
                c if c == quote && !escaped => {
                    closed = true;
                    self.advance();
                    break;
                },
                c if c == '\\' && !escaped => {
                    escaped = true;
                },
                '"' | '\'' => {
                    value.push(current);
                    escaped = false;
                },
                _ if escaped => {
                    let esc = self.escape_current()?;
                    value.push(esc);
                    escaped = false;
                },
                _ => {
                    value.push(current);
                    escaped = false;
                }
            }

            self.advance();
        }

        if !closed {
            Err(self.error("unexpected end of file", "closing quotes"))?
        }

        Ok(Token {
            kind: TokenKind::Literal(Literal::String(value)),
            position: self.position,
        })
    }

    /// Parse escape sequence, without the leading backslash
    /// Does not advance the position
    fn escape_current(&mut self) -> Result<char, String> {
        let current = match self.current() {
            Some(c) => c,
            None => Err(self.error("unexpected end of file", "escaped character"))?
        };

        let escaped = match current {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '"' => '"',
            '\'' => '\'',
            _ => Err(self.error("Invalid escape sequence", "escaped character"))?
        };

        Ok(escaped)
    }

    /// Parse identifier or keyword token
    fn identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut value = String::new();

        if self.match_next('0'..='9') {
            Err(self.error("unexpected digit", "identifier or keyword"))?
        }

        while let Some(current) = self.current() {
            match current {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => value.push(current),
                _ => break,
            }

            self.advance();
        }

        if value.len() == 0 {
            Err(self.error("unexpected character", "identifier or keyword"))?
        }

        let keyword = match value.as_str() {
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "function" | "fn" => Some(Keyword::Function),
            "return" => Some(Keyword::Return),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "let" => Some(Keyword::Let),
            "const" => Some(Keyword::Const),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            _ => None,
        };

        let kind = match keyword {
            Some(keyword) => TokenKind::Keyword(keyword),
            None => TokenKind::Identifier(value),
        };
        
        Ok(Token {
            kind,
            position: self.position,
        }) 
    }

    /// Parse number token
    fn number(&mut self) -> Result<Token, String> {
        let mut value = String::new();
        let mut is_float = false;
        let mut prev_underscore = false;
        
        while let Some(current) = self.current() {
            match current {
                'a'..='z' | 'A'..='Z' => {
                    Err(self.error("unexpected character", "number"))?
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

        if prev_underscore { Err(self.error("unexpected underscore", "number"))? } 
        if value.len() == 0 { Err(self.error("unexpected character", "number"))? }

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
