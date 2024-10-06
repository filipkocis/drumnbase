use std::ops::RangeBounds;

use super::token::{Token, TokenKind, Symbol, Operator, Literal, Keyword, QueryKeyword, SDLKeyword};

pub struct Tokenizer {
    input: String,
    position: usize,
    token_start: usize,
    line: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Tokenizer {
        Tokenizer {
            input,
            position: 0,
            token_start: 0,
            line: 0,
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
        if let Some('\n') = self.current() {
            self.line += 1;
        }
        self.position += 1;
    }

    /// Update token start position
    fn mark_start(&mut self) {
        self.token_start = self.position;
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

    /// Returns new token struct wrapped in Ok
    /// utility function to avoid writing `Ok(Token::new(...))`
    fn ok_token(&self, kind: TokenKind) -> Result<Token, String> {
        Ok(Token::new(kind, self.token_start, self.position, self.line))
    }

    /// Parse the next token
    fn token(&mut self) -> Result<Token, String> {
        if self.eof() {
            return self.ok_token(TokenKind::EOF)
        }

        let current = self.current().unwrap();
        match current {
            c if c == '/' && self.is_next('/') => {
                self.consume_comment();
                return self.token();
            }
            ' ' | '\t' | '\n' => {
                self.advance();
                return self.token();
            },
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(),
            '"' | '\'' => self.string(),
            _ => self.symbol_or_operator(),
        }
    }

    /// Parse symbol or operator token
    fn symbol_or_operator(&mut self) -> Result<Token, String> {
        let current = self.current().expect("unexpected end of file, expected symbol or operator");

        self.mark_start();
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
            '{' => Some(Symbol::LeftBrace),
            '}' => Some(Symbol::RightBrace),
            '[' => Some(Symbol::LeftBracket),
            ']' => Some(Symbol::RightBracket),
            _ => match (current, self.next()) {
                ('-', Some('>')) => {
                    self.advance();
                    Some(Symbol::Arrow)
                },
                _ => None,
            }
        };

        if let Some(symbol) = symbol {
            self.advance();
            return self.ok_token(TokenKind::Symbol(symbol));
        }

        let adv_ret = |s: &mut Tokenizer, op| {
            s.advance();
            op
        };

        self.mark_start();
        let operator = match current {
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
                ('*', Some('*')) => adv_ret(self, Some(Operator::Power)),
                ('*', _) => Some(Operator::Multiply),

                ('/', Some('=')) => adv_ret(self, Some(Operator::DivideAssign)),

                ('%', Some('=')) => adv_ret(self, Some(Operator::ModulusAssign)),
                ('%', _) => Some(Operator::Modulus),

                _ => None,
            }
        };

        if let Some(operator) = operator {
            self.advance();
            return self.ok_token(TokenKind::Operator(operator))
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

        self.mark_start();
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

        Ok(Token::new(
                TokenKind::Literal(Literal::String(value)),
                self.token_start,
                self.position - 1, // exclude closing quote
                self.line,
        ))
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

        if matches!(self.current(), Some('0'..='9')) {
            Err(self.error("unexpected digit", "identifier or keyword"))?
        }

        self.mark_start();
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
            "for" => Some(Keyword::For),
            "function" | "fn" => Some(Keyword::Function),
            "return" => Some(Keyword::Return),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "let" => Some(Keyword::Let),
            "const" => Some(Keyword::Const),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "on" => Some(Keyword::On),
            "to" => Some(Keyword::To),
            _ => None
        };

        if keyword.is_none() {
            if let Some(token) = self.query(&value) {
                return Ok(token)
            }

            if let Some(token) = self.sdl(&value) {
                return Ok(token)
            }
        }

        let kind = match keyword {
            Some(keyword) => TokenKind::Keyword(keyword),
            None => TokenKind::Identifier(value),
        };
        
        self.ok_token(kind)
    }

    /// Parse query keywords
    fn query(&self, value: &str) -> Option<Token> {
        let query_keyword = match value {
            "query" => QueryKeyword::Query,
            "as" => QueryKeyword::As,
            "join" => QueryKeyword::Join,

            "select" => QueryKeyword::Select,
            "insert" => QueryKeyword::Insert,
            "update" => QueryKeyword::Update,
            "delete" => QueryKeyword::Delete,

            "where" => QueryKeyword::Where,
            "order" => QueryKeyword::Order,
            "limit" => QueryKeyword::Limit,
            "offset" => QueryKeyword::Offset,
            "exclude" => QueryKeyword::Exclude,
    
            _ => return None
        };

        self.ok_token(TokenKind::Query(query_keyword)).ok()
    }

    /// Parse sdl keywords
    fn sdl(&self, value: &str) -> Option<Token> {
        let sdl_keyword = match value {
            "database" => SDLKeyword::Database,
            "table" => SDLKeyword::Table,
            "column" => SDLKeyword::Column,
            "policy" => SDLKeyword::Policy,
            "user" => SDLKeyword::User,
            "role" => SDLKeyword::Role,

            "create" => SDLKeyword::Create,
            "drop" => SDLKeyword::Drop,
            "alter" => SDLKeyword::Alter,

            "default" => SDLKeyword::Default,
            "unique" => SDLKeyword::Unique,
            "required" => SDLKeyword::Required,

            "key" => SDLKeyword::Key,
            "references" => SDLKeyword::References,
    
            "grant" => SDLKeyword::Grant,
            "revoke" => SDLKeyword::Revoke,
            "connect" => SDLKeyword::Connect,
            "execute" => SDLKeyword::Execute,

            _ => return None
        };

        self.ok_token(TokenKind::SDL(sdl_keyword)).ok()
    }

    /// Parse number token
    fn number(&mut self) -> Result<Token, String> {
        let mut value = String::new();
        let mut is_float = false;
        let mut prev_underscore = false;
        
        self.mark_start();
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

        if prev_underscore { Err(self.error("underscore needs to preced a number", "number"))? } 
        if value.len() == 0 { Err(self.error("unexpected character", "number"))? }

        let literal = if is_float {
            Literal::Float(value)
        } else {
            Literal::Int(value)
        };

        self.ok_token(TokenKind::Literal(literal))
    }

    fn consume_comment(&mut self) {
        while let Some(current) = self.current() {
            if current == '\n' { break; }
            self.advance();
        }
    }
}
