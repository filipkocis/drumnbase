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

    pub fn parse(&mut self) -> Result<Node, String> {
        let mut statements = vec![];

        while let Some(token) = self.current() {
            if token.kind == TokenKind::EOF {
                break;
            } else {
                statements.push(self.statement()?)
            }
            self.advance();
        }

        Ok(Node::Block(statements))
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


    fn statement(&mut self) -> Result<Node, String> {
        let token = self.current_token()?;

        match token.kind {
            TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Keyword(_) => self.keyword(),
            _ => self.expression(),
        }
    }

    fn keyword(&mut self) -> Result<Node, String> {
        let token = self.current_token()?;

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
                _ => todo!("keyword")
            }
        } else {
            Err("Expected keyword".to_string())
        }
    }

    fn if_statement(&mut self) -> Result<Node, String> {
        self.expect(TokenKind::Keyword(Keyword::If))?;

        let condition = self.expression()?;
        let then_block = self.expression()?; 

        let else_block = if let Some(token) = self.current() {
            if token.kind == TokenKind::Keyword(Keyword::Else){
                self.advance();
                Some(self.expression()?)
            } else {
                None
            }
        } else { 
            None
        };

        Ok(Node::Statement(Statement::If {
            condition: Box::new(condition),
            then_block: Box::new(then_block),        
            else_block: else_block.map(Box::new)
        }))
    }

    fn block(&mut self) -> Result<Node, String> {
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

    fn expression(&mut self) -> Result<Node, String> {
        let token = self.current_token()?;

        match token.kind {
            TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Literal(_) => self.literal(),
            TokenKind::Symbol(_) => self.symbol(), 
            _ => todo!("expression")
        }
    }

    fn literal(&mut self) -> Result<Node, String> {
        let token = self.current_token()?;

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
}
