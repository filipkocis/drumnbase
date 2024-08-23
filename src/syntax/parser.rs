use std::fmt::Debug;

use crate::syntax::token::TokenKind;

use super::{token::{Token, Keyword, Symbol, Literal}, ast::{Node, Statement, Number, self}};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ASTError {
    message: String,
    token: Token,
}

impl ASTError {
    fn new(message: String, token: Token) -> Self {
        Self { message, token }
    }
}

#[derive(Debug)]
pub struct ParserError {
    errors: Vec<ASTError>,
}

impl ParserError {
    pub fn new(errors: Vec<ASTError>) -> Self {
        Self { errors }
    }

    pub fn print(&self) {
        for error in &self.errors {
            println!("Error: {}", error.message);
        }
    }

    pub fn highlight(&self, input: &str) {
        for error in &self.errors {
            println!("Error: {}", error.message);
            println!("{}", self.highlight_token(input, &error.token));
        }
    }

    fn highlight_token(&self, input: &str, token: &Token) -> String {
        let mut result = String::new();
        let start = token.index.start;
        let end = token.index.end;

        for (i, c) in input.chars().enumerate() {
            if i == start { result.push_str("\u{1b}[30;43m") } 
            if i == start && i == end { result.push_str(" ") } // EOF
            if i == end { result.push_str("\u{1b}[0m") } 

            result.push(c);
        }

        result.push_str("\u{1b}[0m");
        result       
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Node, ParserError> {
        let mut statements = vec![];
        let mut errors = vec![];
        let mut previous_current;

        while let Some(token) = self.current() {
            if token.kind == TokenKind::EOF {
                break;
            } else {
                previous_current = self.current;
                match self.statement() {
                    Ok(statement) => statements.push(statement),
                    Err(error) => {
                        if previous_current == self.current {
                            // Prevent infinite loop
                            self.advance();
                        }
                        errors.push(error);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(Node::Block(statements))
        } else {
            Err(ParserError::new(errors))
        }
    }

    /// Expects the current token to be of the specified kind, consumes and returns it.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ASTError> {
        self.advance();
        let token = self.previous();

        match token {
            Some(_) if self.previous().unwrap().kind == kind => Ok(self.previous().unwrap()),
            _ => Err(self.expected(kind))
        }
    }

    fn expected(&mut self, expected: impl Debug) -> ASTError {
        let current = self.current().cloned();
        self.advance();
        let end = self.tokens.last().expect("Empty token list").index.end;

        if let Some(token) = current {
            ASTError::new(
                format!("Expected {:?} but found {:?}", expected, token.kind),
                token
            )
        } else {
            ASTError::new(
                format!("Expected {:?} but found None", expected),
                Token::new(TokenKind::EOF, end, end)
            )
        }
    }

    /// Returns the current token, or an error if there is none.
    fn current_token(&mut self, expected: impl Debug) -> Result<&Token, ASTError> {
        match self.current() {
            Some(_) => Ok(self.current().unwrap()),
            None => Err(self.expected(expected)) 
        }
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


    fn statement(&mut self) -> Result<Node, ASTError> {
        let token = self.current_token("statement")?;

        match token.kind {
            // TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Keyword(_) => self.keyword(),
            _ => self.expression(),
        }
    }

    fn keyword(&mut self) -> Result<Node, ASTError> {
        let token = self.current_token("keyword")?;

        if let TokenKind::Keyword(ref keyword) = token.kind {
            match keyword {
                Keyword::If => self.if_statement(),
                // Keyword::Else => self.else_statement(),
                // Keyword::While => self.while_statement(),
                // Keyword::For => self.for_statement(),
                // Keyword::Function => self.function_declaration_statement(),
                // Keyword::Return => self.return_statement(),
                Keyword::Break => Ok(Node::Statement(Statement::Break)),
                Keyword::Continue => Ok(Node::Statement(Statement::Continue)),
                // Keyword::Let => self.let_statement(),
                // Keyword::Const => self.const_statement(),
                Keyword::True | Keyword::False => Ok(Node::Literal(ast::Literal::Boolean(keyword == &Keyword::True))),
                Keyword::Null => Ok(Node::Literal(ast::Literal::Null)),
                // _ => todo!("keyword")
                _ => Err(self.expected("keyword"))?
            }
        } else {
            Err(self.expected("keyword"))?
        }
    }

    fn if_statement(&mut self) -> Result<Node, ASTError> {
        self.expect(TokenKind::Keyword(Keyword::If))?;

        let condition = self.expression()?;
        let then_block = self.expression()?; 

        let else_block = match self.current() {
            Some(token) => match token.kind {
                TokenKind::Keyword(Keyword::Else) => {
                    self.advance();
                    if let Some(token) = self.current() {
                        if token.kind == TokenKind::Keyword(Keyword::If) {
                            return self.if_statement();
                        } 
                    }
                    Some(self.expression()?) 
                },
                _ => None
            }
            _ => None
        };

        Ok(Node::Statement(Statement::If {
            condition: Box::new(condition),
            then_block: Box::new(then_block),        
            else_block: else_block.map(Box::new)
        }))
    }

    fn block(&mut self) -> Result<Node, ASTError> {
        // TODO: implement block error handling like in parse(), eat brace after error
        self.expect(TokenKind::Symbol(Symbol::LeftBrace))?;

        let mut statements = Vec::new();

        while let Some(token) = self.current() {
            if token.kind == TokenKind::Symbol(Symbol::RightBrace) {
                break;
            } else {
                statements.push(self.statement()?);
            }
        }

        self.expect(TokenKind::Symbol(Symbol::RightBrace))?;
        Ok(Node::Block(statements))
    }

    fn expression(&mut self) -> Result<Node, ASTError> {
        let token = self.current_token("expression")?;

        match token.kind {
            // TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Literal(_) => self.literal(),
            TokenKind::Symbol(_) => self.symbol(), 
            // _ => todo!("expression")
            _ => Err(self.expected("expression"))?
        }
    }

    fn literal(&mut self) -> Result<Node, ASTError> {
        let token = self.current_token("literal")?;

        let literal = if let TokenKind::Literal(ref literal) = token.kind {
            match literal {
                Literal::Int(value) => ast::Literal::Number(Number::Int(value.parse().unwrap())),            
                Literal::Float(value) => ast::Literal::Number(Number::Float(value.parse().unwrap())),            
                Literal::String(value) => ast::Literal::String(value.clone()),            
            }
        } else {
            Err(self.expected("literal"))?
        };

        self.advance();
        Ok(Node::Literal(literal))
    }

    fn symbol(&mut self) -> Result<Node, ASTError> {
        let token = self.current_token("symbol")?;
        
        if let TokenKind::Symbol(ref symbol) = token.kind {
            match symbol {
                Symbol::LeftParenthesis => self.group(),
                Symbol::LeftBrace => self.block(),
                Symbol::LeftBracket => self.array(),
                // _ => todo!("symbol")
                _ => Err(self.expected("symbol"))?
            }
        } else {
            Err(self.expected("symbol"))?
        }
    }

    fn group(&mut self) -> Result<Node, ASTError> {
        // TODO: implement group
        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
        let expression = self.expression()?;
        self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?;
        Ok(expression)
    }

    fn array(&mut self) -> Result<Node, ASTError> {
        self.expect(TokenKind::Symbol(Symbol::LeftBracket))?;
        let mut elements = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::RightBracket) => break,
                TokenKind::Symbol(Symbol::Comma) => {
                    self.advance();
                    continue;
                },
                _ => elements.push(self.expression()?)
            }
        }

        self.expect(TokenKind::Symbol(Symbol::RightBracket))?;
        Ok(Node::Literal(ast::Literal::Array(elements)))
    }
}
