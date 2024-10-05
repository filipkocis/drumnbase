use std::fmt::Debug;

use crate::{syntax::token::{TokenKind, SDLKeyword}, basics::{Column, column::{ColumnType, NumericType, TextType, TimestampType}}, auth::{RlsPolicy, RlsAction}};

use super::{token::{Token, Keyword, Symbol, Literal, Operator, QueryKeyword}, ast::{Node, Statement, Number, self, Expression, Type, SelectQuery, InsertQuery, UpdateQuery, DeleteQuery, CreateSDL}};

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

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug)]
pub struct ParserError {
    errors: Vec<ASTError>,
}

impl From<ASTError> for ParserError {
    fn from(error: ASTError) -> Self {
        Self { errors: vec![error] }
    }
}

impl ParserError {
    pub fn errors(&self) -> &Vec<ASTError> {
        &self.errors
    }

    pub fn add(&mut self, error: ASTError) {
        self.errors.push(error);
    }

    pub fn extend(&mut self, parser_error: ParserError) {
        self.errors.extend(parser_error.errors)
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn empty() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn from(error: ASTError) -> Self {
        Self { errors: vec![error] }
    }

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
            println!("Error on line {}: {}", error.token.line + 1, error.message);
            println!("{}", self.highlight_token(input, &error.token));
        }
    }

    fn highlight_token(&self, input: &str, token: &Token) -> String {
        let mut result = String::new();
        let start = token.index.start;
        let end = token.index.end;

        let mut offset = 0;
        let mut line_start = 0; 

        let lines = input
            .split('\n')
            .collect::<Vec<_>>()
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                let low_bound = if token.line < 3 { 0 } else { token.line - 3 };
                match i {
                    _ if i < low_bound => {
                        line_start += 1;
                        offset += line.len() + 1;
                        None
                    },
                    _ if i > token.line + 3 => None,
                    _ => Some(line.to_string())
                } 
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut current_line = 0;
        for (idx, c) in lines.chars().enumerate() {
            if lines.chars().nth(idx.saturating_sub(1)) == Some('\n') || idx == 0 {
                current_line += 1;
                result.push_str(format!("{: >4} | ", line_start + current_line).as_str())
            }

            let i = idx + offset;

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
        let mut parser_error = ParserError::empty();
        let mut previous_current;

        while let Some(token) = self.current() {
            if token.kind == TokenKind::EOF { break; }
            let mut added_error = false;

            previous_current = self.current;
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    if previous_current == self.current {
                        // Prevent infinite loop
                        self.advance();
                    }
                    parser_error.extend(error);
                    added_error = true;
                }
            }

            match self.current() {
                Some(token) if token.kind == TokenKind::Symbol(Symbol::Semicolon) => self.advance(),
                Some(token) if token.kind == TokenKind::EOF => break,
                Some(_) if added_error => continue,
                Some(_) => parser_error.add(self.missing(TokenKind::Symbol(Symbol::Semicolon))),
                _ => break
            }
        }

        if parser_error.is_empty() {
            Ok(Node::Block(statements))
        } else {
            Err(parser_error)
        }
    }

    /// Expects the current token to be of the specified kind, consumes and returns it.
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ASTError> {
        match self.current() {
            Some(token) if token.kind == kind => {
                self.advance();
                return Ok(self.previous().unwrap())
            }
            _ => Err(self.expected(kind))
        }
    }

    fn expected(&mut self, expected: impl Debug) -> ASTError {
        let current = self.current().cloned();
        self.advance();

        if let Some(token) = current {
            ASTError::new(
                format!("Expected {:?} but found {:?}", expected, token.kind),
                token
            )
        } else {
            ASTError::new(
                format!("Expected {:?} but found None", expected),
                self.eof_default()
            )
        }
    }

    fn expected_node(&mut self, expected: impl Debug, node: impl Debug) -> ASTError {
        let current = self.current().cloned().unwrap_or(self.eof_default());
        self.advance();

        ASTError::new(
            format!("Expected {:?} but found {:?}", expected, node),
            current
        )
    }

    fn eof_default(&self) -> Token {
        let end = self.tokens.last().expect("Empty token list").index.end;
        let line = self.tokens.last().unwrap().line;
        Token::new(TokenKind::EOF, end, end, line)
    }

    fn missing(&self, missing: impl Debug) -> ASTError {
        let token = self.previous().cloned().unwrap_or(self.eof_default());
        ASTError::new(
            format!("Missing {:?}", missing),
            token
        )
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

    #[allow(dead_code)]
    fn next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) {
        self.current += 1;
    }


    fn statement(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("statement")?;

        match token.kind {
            // TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Keyword(_) => self.keyword(),
            TokenKind::Query(_) => self.query(),
            TokenKind::SDL(_) => self.sdl(),
            _ => self.expression(),
        }
    }

    fn keyword(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("keyword")?;

        let node = if let TokenKind::Keyword(ref keyword) = token.kind {
            match keyword {
                Keyword::If => return self.if_statement(),
                // Keyword::Else => self.else_statement(),
                Keyword::While => return self.while_statement(),
                Keyword::For => return self.for_statement(),
                Keyword::Function => return self.function_declaration_statement(),
                Keyword::Return => return self.return_statement(),
                Keyword::Break => Node::Statement(Statement::Break),
                Keyword::Continue => Node::Statement(Statement::Continue),
                Keyword::Let => return self.let_statement(),
                // Keyword::Const => self.const_statement(),
                k if k.is_literal() => return self.keyword_literal(),
                _ => Err(self.expected("valid keyword"))?
            }
        } else {
            Err(self.expected("keyword"))?
        };

        self.advance(); // consume keyword created with Node::Kind
        Ok(node)
    }

    fn for_statement(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;

        let initializer = self.statement()?;
        self.expect(TokenKind::Symbol(Symbol::Semicolon))?;

        let condition = self.expression()?;
        self.expect(TokenKind::Symbol(Symbol::Semicolon))?;

        let action = self.statement()?;
        self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?;

        let block = self.block()?;
        
        Ok(Node::Statement(Statement::For { 
            initializer: Box::new(initializer),
            condition: Box::new(condition),
            action: Box::new(action),
            block: Box::new(block), 
        }))
    }

    fn while_statement(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Keyword(Keyword::While))?;

        let condition = self.expression()?;
        let block = self.block()?;

        Ok(Node::Statement(Statement::While { 
            condition: Box::new(condition), 
            block: Box::new(block) 
        }))
    }

    fn return_statement(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Keyword(Keyword::Return))?;

        let value = match self.current() {
            Some(token) if token.kind == TokenKind::Symbol(Symbol::Semicolon) => {
                self.advance();
                Node::Literal(ast::Literal::Null)
            },
            _ => self.expression()?,
        };

        Ok(Node::Statement(Statement::Return(Box::new(value))))
    }

    fn let_statement(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Keyword(Keyword::Let))?;

        let name =  match self.current_token("identifier")? {
            Token { kind: TokenKind::Identifier(name), .. } => name.clone(),
            _ => Err(self.expected("identifier"))?
        };
        self.advance();
        self.expect(TokenKind::Operator(Operator::Assign))?;

        let value = self.statement()?;

        Ok(Node::Statement(Statement::Let { name, value: Box::new(value) }))
    }

    fn keyword_literal(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("keyword literal")?;

        let node = if let TokenKind::Keyword(ref keyword) = token.kind {
            match keyword {
                Keyword::True | Keyword::False => Node::Literal(ast::Literal::Boolean(keyword == &Keyword::True)),
                Keyword::Null => Node::Literal(ast::Literal::Null),
                _ => Err(self.expected("keyword literal"))?
            }
        } else {
            Err(self.expected("keyword literal"))?
        };

        self.advance();
        Ok(node)
    }

    fn function_declaration_statement(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Keyword(Keyword::Function))?;

        let name = match self.current() {
            Some(Token { kind: TokenKind::Identifier(name), .. }) => name.clone(), 
            _ => Err(self.expected("function name"))?
        };
        self.advance();

        let parameters = self.parameters()?; 
        let return_type = self.return_type()?;
        let block = self.block()?;

        Ok(Node::Statement(Statement::Function { name, parameters, return_type, block: Box::new(block) }))
    }

    fn parameters(&mut self) -> Result<Vec<(String, Type)>, ParserError> {
        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
        let mut parser_error = ParserError::empty();
        let mut parameters = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::RightParenthesis) => break,
                TokenKind::Symbol(Symbol::Comma) => {
                    self.advance();
                    continue;
                },
                TokenKind::Identifier(ref name) => {
                    let name = name.clone();
                    self.advance(); 
                    match self.parameter_type() {
                        Ok(parameter_type) => parameters.push((name, parameter_type)),
                        Err(error) => parser_error.add(error)
                    }
                },
                _ => {
                    parser_error.add(self.expected("parameter"));
                    return Err(parser_error)
                }
            }
        } 

        if let Err(err) = self.expect(TokenKind::Symbol(Symbol::RightParenthesis)) {
            parser_error.add(err)
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(parameters)
    }

    fn type_declaration(&mut self) -> Result<Type, ASTError> {
        let token = self.current_token("type")?;

        let declared_type = match token.kind {
            TokenKind::Identifier(ref identifier) => {
                let declared_type = match identifier.as_str() {
                    "int" => Type::Int,
                    "uint" => Type::UInt,
                    "float" => Type::Float,
                    "string" => Type::String,
                    "bool" => Type::Boolean,
                    _ => Err(self.expected("valid type"))?
                };
                self.advance();
                declared_type
            },
            TokenKind::Operator(Operator::Multiply) => {
                self.advance();
                Type::Pointer(Box::new(self.type_declaration()?))
            }
            TokenKind::Symbol(Symbol::LeftBracket) => {
                self.advance();
                let declared_type = self.type_declaration()?;
                self.expect(TokenKind::Symbol(Symbol::RightBracket))?;
                Type::Array(Box::new(declared_type))
            },
            _ => Err(self.expected("valid type"))?
        };

        Ok(declared_type)
    }

    fn parameter_type(&mut self) -> Result<Type, ASTError> {
        self.expect(TokenKind::Symbol(Symbol::Colon))?;
        self.type_declaration()
    }

    fn return_type(&mut self) -> Result<Type, ASTError> {
        if let Some(token) = self.current() {
            if token.kind == TokenKind::Symbol(Symbol::LeftBrace) {
                return Ok(Type::Void)
            }
        }

        self.expect(TokenKind::Symbol(Symbol::Arrow))?;
        self.type_declaration()
    }

    fn if_statement(&mut self) -> Result<Node, ParserError> {
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

    fn block(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Symbol(Symbol::LeftBrace))?;
        let mut parser_error = ParserError::empty();
        let mut statements = Vec::new();

        while let Some(token) = self.current() {
            let mut added_error = false;
            match token.kind {
                TokenKind::Symbol(Symbol::RightBrace) => break,
                _ => {
                    match self.statement() {
                        Ok(statement) => statements.push(statement),
                        Err(error) => {
                            parser_error.extend(error);
                            added_error = true;
                        }
                    }
                }
            }

            match self.current() {
                Some(token) if token.kind == TokenKind::Symbol(Symbol::Semicolon) => self.advance(),
                Some(token) if token.kind == TokenKind::Symbol(Symbol::RightBrace) => break,
                Some(token) if token.kind == TokenKind::EOF => break,
                Some(_) if added_error => continue,
                Some(_) => parser_error.add(self.missing(TokenKind::Symbol(Symbol::Semicolon))),
                None => break
           }  
        }

        if let Err(err) = self.expect(TokenKind::Symbol(Symbol::RightBrace)) {
            parser_error.add(err);
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(Node::Block(statements))
    }

    fn expression(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("expression")?;

        // expression can start with unary operator
        if let TokenKind::Operator(ref operator) = token.kind {
            if operator.is_unary() {
                let ast_operator = operator.to_ast_operator();
                self.advance();
                let right = self.expression()?;
                return Ok(Node::Expression(Expression::Unary { operator: ast_operator, right: Box::new(right) }))
            }
        }

        // expression can start with a literal, symbol or identifier
        let node = match token.kind {
            // TokenKind::EOF => Err("Unexpected EOF".to_string()),
            TokenKind::Literal(_) => self.literal()?,
            TokenKind::Symbol(_) => self.symbol()?, 
            TokenKind::Identifier(_) => self.identifier()?, 
            TokenKind::Keyword(ref c) if c.is_literal() => self.keyword_literal()?,
            // _ => todo!("expression")
            _ => Err(self.expected("expression"))?
        };

        // expression can be followed by increment or decrement operator
        if let Some(token) = self.current() {
            if let TokenKind::Operator(ref operator) = token.kind {
                if operator.is_unary() {
                    let ast_operator = operator.to_ast_operator();
                    self.advance();
                    return Ok(Node::Expression(Expression::Unary { operator: ast_operator, right: Box::new(node) }))
                }
            }
        }

        // expression can be followed by an operator
        if let Some(token) = self.current() {
            match token.kind {
                TokenKind::Operator(ref operator) => {
                    if operator.is_assigment() {
                        return self.assignment(node)
                    } else if operator.is_binary() {
                        return self.binary(node)
                    } else {
                        Err(self.expected("valid operator"))?
                    }
                },
                _ => { }
            }
        }

        Ok(node)
    }

    fn assignment(&mut self, left: Node) -> Result<Node, ParserError> {
        let token = self.current_token("assignment operator")?.clone(); 

        let identifier = match left {
            Node::Literal(ast::Literal::Identifier(ref name)) => name.clone(),
            _ => Err(self.expected_node("identifier before assignment", &left))? 
        };

        if let TokenKind::Operator(ref operator) = token.kind {
            if !operator.is_assigment() { Err(self.expected("assignment operator"))? }
            self.advance();

            let ast_operator = operator.to_ast_operator();
            let right = self.expression()?;
            let value = match ast_operator {
                ast::Operator::Assign => right,
                _ => {
                    let ast_operator = match ast_operator {
                        ast::Operator::AddAssign => ast::Operator::Add,
                        ast::Operator::SubAssign => ast::Operator::Sub,
                        ast::Operator::MulAssign => ast::Operator::Mul,
                        ast::Operator::DivAssign => ast::Operator::Div,
                        ast::Operator::ModAssign => ast::Operator::Mod,
                        ast::Operator::PowAssign => ast::Operator::Pow,
                        _ => Err(self.expected("assignment operator"))?
                    };
                    Node::Expression(Expression::Binary { left: Box::new(left), operator: ast_operator, right: Box::new(right) })
                }
            };

            Ok(Node::Statement(Statement::Assignment { name: identifier, value: Box::new(value) }))
        } else {
            Err(self.expected("assignment operator"))?
        }
    }

    fn binary(&mut self, left: Node) -> Result<Node, ParserError> {
        let token = self.current_token("binary operator")?.clone();

        if let TokenKind::Operator(ref operator) = token.kind {
            if !operator.is_binary() { Err(self.expected("binary operator"))? }
            self.advance();

            let ast_operator = operator.to_ast_operator();
            let right = self.expression()?;

            Ok(Node::Expression(Expression::Binary { left: Box::new(left), operator: ast_operator, right: Box::new(right) }))
        } else {
            Err(self.expected("binary operator"))?
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

    fn symbol(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("symbol")?;
        
        if let TokenKind::Symbol(ref symbol) = token.kind {
            match symbol {
                Symbol::LeftParenthesis => self.group(),
                Symbol::LeftBrace => self.block(),
                Symbol::LeftBracket => self.array(),
                // _ => todo!("symbol")
                _ => Err(self.expected("valid symbol"))?
            }
        } else {
            Err(self.expected("symbol"))?
        }
    }

    fn group(&mut self) -> Result<Node, ParserError> {
        // TODO: implement group
        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
        let expression = self.expression()?;
        self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?;
        Ok(expression)
    }

    fn array(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Symbol(Symbol::LeftBracket))?;
        let mut parser_error = ParserError::empty();
        let mut elements = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::RightBracket) => break,
                TokenKind::Symbol(Symbol::Comma) => {
                    self.advance();
                    continue;
                },
                _ => match self.expression() {
                    Ok(expression) => elements.push(expression),
                    Err(error) => parser_error.extend(error)
                }
            }
        }

        if let Err(err) = self.expect(TokenKind::Symbol(Symbol::RightBracket)) {
            parser_error.add(err);
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(Node::Literal(ast::Literal::Array(elements)))
    }

    fn identifier_name(&mut self) -> Result<String, ASTError> {
        let token = self.current_token("identifier")?;

        let identifier = if let TokenKind::Identifier(ref identifier) = token.kind {
            identifier.clone()
        } else {
            Err(self.expected("identifier"))?
        };

        self.advance();
        Ok(identifier)
    }
    
    fn identifier(&mut self) -> Result<Node, ParserError> {
        let identifier = self.identifier_name()?;

        match self.current() {
            Some(token) => match token.kind {
                TokenKind::Symbol(Symbol::LeftParenthesis) => {
                    let arguments = self.arguments()?;
                    Ok(Node::Expression(Expression::Call { name: identifier, arguments }))
                },
                // TokenKind::Symbol(Symbol::Period) => {
                //     self.advance();
                //     let property = self.identifier()?;
                //     Ok(Node::Member { name: identifier, member: Box::new(property) })
                // },
                _ => Ok(Node::Literal(ast::Literal::Identifier(identifier)))
            },
            None => Ok(Node::Literal(ast::Literal::Identifier(identifier)))
        }
    }

    fn arguments(&mut self) -> Result<Vec<Node>, ParserError> {
        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
        let mut parser_error = ParserError::empty();
        let mut arguments = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::RightParenthesis) => break,
                TokenKind::Symbol(Symbol::Comma) => {
                    self.advance();
                    continue;
                }
                _ => match self.expression() {
                    Ok(expression) => arguments.push(expression),
                    Err(error) => parser_error.extend(error)
                }
            }
        }

        if let Err(err) = self.expect(TokenKind::Symbol(Symbol::RightParenthesis)) {
            parser_error.add(err);
        }

        if !parser_error.is_empty() {
            return Err(parser_error);
        }

        Ok(arguments)
    }
}

// QUERY
impl Parser {
    fn query(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Query))?;
        
        let table_name = self.string_or_identifier()?;

        match self.current() {
            Some(token) => match token.kind {
                TokenKind::Query(QueryKeyword::Select) => self.select_query(table_name),
                TokenKind::Query(QueryKeyword::Insert) => self.insert_query(table_name),
                TokenKind::Query(QueryKeyword::Update) => self.update_query(table_name),
                TokenKind::Query(QueryKeyword::Delete) => self.delete_query(table_name),
                _ => Err(self.expected("valid query type"))?
            },
            _ => Err(self.expected("query type"))?
        }
    }

    fn string_or_identifier(&mut self) -> Result<String, ASTError> {
        let token = self.current_token("string or identifier")?;

        let value = match &token.kind {
            TokenKind::Literal(Literal::String(value)) |
            TokenKind::Identifier(value) => value.clone(),
            _ => Err(self.expected("string or identifier"))? 
        };
        self.advance();

        Ok(value)
    }

    fn delete_query(&mut self, table_name: String) -> Result<Node, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Delete))?;
        let query = DeleteQuery {
            table: table_name,
            where_clause: Some(self.query_where()?)
        };

        Ok(Node::Query(ast::Query::Delete(query)))
    }

    fn update_query(&mut self, table_name: String) -> Result<Node, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Update))?;
        let mut parser_error = ParserError::empty();

        let key_values = match self.query_key_values() {
            Ok(key_values) => key_values,
            Err(error) => {
                parser_error.extend(error);
                Vec::new()
            }
        };

        let where_clause = match self.query_where() {
            Ok(where_clause) => where_clause,
            Err(error) => {
                parser_error.extend(error);
                return Err(parser_error)
            }
        };

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        let query = UpdateQuery {
            table: table_name,
            key_values,
            where_clause: Some(where_clause),
        };

        Ok(Node::Query(ast::Query::Update(query)))
    }

    fn insert_query(&mut self, table_name: String) -> Result<Node, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Insert))?;
        let query = InsertQuery {
            table: table_name,
            key_values: self.query_key_values()?,
        };

        Ok(Node::Query(ast::Query::Insert(query)))
    }

    fn query_key_values(&mut self) -> Result<Vec<(String, Node)>, ParserError> {
        let mut parser_error = ParserError::empty();
        let mut key_values = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Query(_) => break,
                TokenKind::Symbol(Symbol::Semicolon) => break,
                TokenKind::EOF => break,
                TokenKind::Identifier(ref key) |
                TokenKind::Literal(Literal::String(ref key)) => {
                    let key = key.clone();
                    self.advance();
                    if let Err(error) = self.expect(TokenKind::Symbol(Symbol::Colon)) {
                        parser_error.add(error);
                        continue;
                    }
                    let value = self.expression()?;
                    key_values.push((key, value))
                },
                _ => parser_error.add(self.expected("key value"))
            }
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(key_values)
    }

    fn select_query(&mut self, table_name: String) -> Result<Node, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Select))?;

        let mut parser_error = ParserError::empty();
        let mut query = SelectQuery {
            table: table_name,
            columns: Vec::new(),
            where_clause: None,
            order: None,
            limit: None,
            offset: None,
            exclude: None,
        };

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::Semicolon) |
                TokenKind::Query(_) => break,
                TokenKind::Symbol(Symbol::Comma) => { self.advance(); },
                TokenKind::Operator(Operator::Multiply) => {
                    self.advance();
                    query.columns.push(Node::Literal(ast::Literal::Identifier("*".to_string())))
                },
                TokenKind::Identifier(_) => {
                    match self.identifier() {
                        Ok(node) => query.columns.push(node),
                        Err(error) => parser_error.extend(error)
                    }
                },
                _ => Err(self.expected("column name"))?
            }
        }

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Query(QueryKeyword::Where) => query.where_clause = Some(self.query_where()?),
                TokenKind::Query(QueryKeyword::Order) => query.order = Some(self.query_order()?),
                TokenKind::Query(QueryKeyword::Limit) => query.limit = Some(self.query_limit()?),
                TokenKind::Query(QueryKeyword::Offset) => query.offset = Some(self.query_offset()?),
                TokenKind::Query(QueryKeyword::Exclude) => query.exclude = Some(self.query_exclude()?),
                _ => break
            }
        };

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(Node::Query(ast::Query::Select(query)))
    }

    fn query_where(&mut self) -> Result<Box<Node>, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Where))?;
        Ok(Box::new(self.expression()?)) 
    } 

    fn query_order(&mut self) -> Result<Box<Node>, ASTError> {
        self.expect(TokenKind::Query(QueryKeyword::Order))?;
        
        let column = self.string_or_identifier()?;
    
        let order = match self.current() {
            Some(Token { kind: TokenKind::Identifier(value), .. }) => {
                match value.as_str() {
                    "asc" => Operator::Increment,
                    "desc" => Operator::Decrement,
                    _ => Err(self.expected("valid order value"))?
                }
            }
            _ => Err(self.expected("order value"))?,
        };
        self.advance();

        Ok(Box::new(Node::Expression(ast::Expression::Unary {
            operator: order.to_ast_operator(),
            right: Box::new(Node::Literal(ast::Literal::Identifier(column)))
        })))
    }

    fn query_limit(&mut self) -> Result<usize, ASTError> {
        self.expect(TokenKind::Query(QueryKeyword::Limit))?;

        match self.current() {
            Some(Token { kind: TokenKind::Literal(Literal::Int(ref value)), .. }) => {
                let value = value.parse().unwrap();
                self.advance();
                return Ok(value)
            },
            _ => Err(self.expected("limit value"))
        }
    }

    fn query_offset(&mut self) -> Result<usize, ASTError> {
        self.expect(TokenKind::Query(QueryKeyword::Offset))?;

        match self.current() {
            Some(Token { kind: TokenKind::Literal(Literal::Int(ref value)), .. }) => {
                let value = value.parse().unwrap();
                self.advance();
                return Ok(value)
            },
            _ => Err(self.expected("offset value"))
        }
    }

    fn query_exclude(&mut self) -> Result<Vec<String>, ParserError> {
        self.expect(TokenKind::Query(QueryKeyword::Exclude))?;

        let mut parser_error = ParserError::empty();
        let mut columns = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Query(_) => break,
                TokenKind::Symbol(Symbol::Semicolon) => { break; },
                TokenKind::Symbol(Symbol::Comma) => { },
                TokenKind::Identifier(ref column) => {
                    columns.push(column.clone());
                    self.advance();
                },
                _ => parser_error.add(self.expected("column name"))
            }

            if let Some(token) = self.current() {
                if token.kind == TokenKind::Symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(columns)
    }
}

// SDL
impl Parser {
    fn sdl(&mut self) -> Result<Node, ParserError> {
        let token = self.current_token("sdl")?;

        let kind = match token.kind {
            TokenKind::SDL(ref sdl) => sdl,
            _ => Err(self.expected("sdl"))?,
        };

        match kind {
            SDLKeyword::Create => self.create(),
            SDLKeyword::Grant => self.grant(),
            // SDLKeyword::Drop => self.drop(),

            _ => Err(self.expected("valid sdl"))?
        }
    }

    fn create(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Create))?;

        let create_sdl = match self.current() {
            Some(token) => match token.kind {
                TokenKind::SDL(SDLKeyword::Database) => self.create_database()?,
                TokenKind::SDL(SDLKeyword::Table) => self.create_table()?,
                TokenKind::SDL(SDLKeyword::Policy) => self.create_policy()?,
                TokenKind::SDL(SDLKeyword::User) => self.create_user()?,
                TokenKind::SDL(SDLKeyword::Role) => self.create_role()?,
                _ => Err(self.expected("valid sdl create object"))?
            },
            None => Err(self.expected("sdl type"))?
        };

        Ok(Node::SDL(ast::SDL::Create(create_sdl)))
    }

    fn create_database(&mut self) -> Result<CreateSDL, ASTError> {
        self.expect(TokenKind::SDL(SDLKeyword::Database))?;
        let name = self.identifier_name()?;
        Ok(CreateSDL::Database { name })
    }

    fn create_table(&mut self) -> Result<CreateSDL, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Table))?;
        let name = self.identifier_name()?;

        self.expect(TokenKind::Symbol(Symbol::LeftBrace))?;
        let mut parser_error = ParserError::empty();
        let mut columns = Vec::new();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::RightBrace) => break,
                TokenKind::Symbol(Symbol::Semicolon) => { self.advance(); },
                TokenKind::Identifier(_) => {
                    let column = self.column_definition()?;
                    columns.push(column)
                }
                _ => parser_error.add(self.expected("column"))
            }
        }

        if let Err(err) = self.expect(TokenKind::Symbol(Symbol::RightBrace)) {
            parser_error.add(err);
        }

        if !parser_error.is_empty() {
            return Err(parser_error)
        }

        Ok(CreateSDL::Table { name, columns })
    }

    fn column_definition(&mut self) -> Result<Column, ParserError> {
        let name = self.identifier_name()?;
        self.expect(TokenKind::Symbol(Symbol::Colon))?;

        let data_type = self.column_type_declaration()?;
        let mut column = Column::new(&name, data_type);
        column.length = column.data_type.len();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::Symbol(Symbol::Semicolon) => break,
                TokenKind::Symbol(Symbol::Comma) => self.advance(),

                TokenKind::SDL(SDLKeyword::Required) => { self.advance(); column.not_null = true },
                TokenKind::SDL(SDLKeyword::Unique) => { self.advance(); column.unique = true },
                TokenKind::SDL(SDLKeyword::Default) => {
                    self.advance();
                    self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
                    column._default = Some(self.expression()?);
                    self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?;
                },
                _ => Err(self.expected("column definition"))?
            }
        }

        Ok(column)
    }

    fn column_type_declaration(&mut self) -> Result<ColumnType, ASTError> {
        let token = self.current_token("column type")?;

        let data_type = match token.kind {
            TokenKind::Identifier(ref identifier) => {
                match identifier.as_str() {
                    "u8" => ColumnType::Numeric(NumericType::IntU8),
                    "u16" => ColumnType::Numeric(NumericType::IntU16),
                    "u32" => ColumnType::Numeric(NumericType::IntU32),
                    "u64" => ColumnType::Numeric(NumericType::IntU64),
                    "i8" => ColumnType::Numeric(NumericType::IntI8),
                    "i16" => ColumnType::Numeric(NumericType::IntI16),
                    "i32" => ColumnType::Numeric(NumericType::IntI32),
                    "i64" => ColumnType::Numeric(NumericType::IntI64),
                    "f32" => ColumnType::Numeric(NumericType::Float32),
                    "f64" => ColumnType::Numeric(NumericType::Float64),

                    "char" => ColumnType::Text(TextType::Char),
                    "variable" => Err(self.expected("fixed size, variable size not supported"))?, 
                    "fixed" => {
                        self.advance();
                        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;

                        let token = self.current_token("fixed size")?;
                        let size = match &token.kind {
                            TokenKind::Literal(Literal::Int(size)) => size.parse().unwrap(),
                            _ => Err(self.expected("fixed size"))? 
                        };
                        self.advance();

                        self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?; 

                        return Ok(ColumnType::Text(TextType::Fixed(size)))
                    },

                    "time" => {
                        self.advance();
                        self.expect(TokenKind::Symbol(Symbol::LeftParenthesis))?;
                        
                        let token = self.current_token("timestamp type")?;
                        let time_type = match &token.kind {
                            TokenKind::Identifier(ref identifier) => {
                                match identifier.as_str() {
                                    "s" => TimestampType::Seconds,
                                    "ms" => TimestampType::Milliseconds,
                                    "us" => TimestampType::Microseconds,
                                    "ns" => TimestampType::Nanoseconds,
                                    _ => Err(self.expected("valid timestamp type"))? 
                                }
                            }
                            _ => Err(self.expected("timestamp type"))? 
                        };
                        self.advance();

                        self.expect(TokenKind::Symbol(Symbol::RightParenthesis))?; 
                        return Ok(ColumnType::Timestamp(time_type))
                    },

                    "bool" => ColumnType::Boolean,

                    _ => Err(self.expected("valid column type"))?
                }
            },
            _ => Err(self.expected("column type"))?
        };
        self.advance();

        Ok(data_type)
    }

    fn literal_string(&mut self) -> Result<String, ASTError> {
        let token = self.current_token("literal string")?;

        let string = match token.kind {
            TokenKind::Literal(Literal::String(ref value)) => value.clone(),
            _ => Err(self.expected("literal string"))?
        };
        self.advance();

        Ok(string)
    }

    fn create_policy(&mut self) -> Result<CreateSDL, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Policy))?;

        let name = self.literal_string()?;
        self.expect(TokenKind::Keyword(Keyword::For))?;
        let table = self.identifier_name()?;
        self.expect(TokenKind::Symbol(Symbol::Period))?;
        
        let action = match self.current_token("policy action")?.kind {
            TokenKind::Query(QueryKeyword::Select) => RlsAction::Select,
            TokenKind::Query(QueryKeyword::Insert) => RlsAction::Insert,
            TokenKind::Query(QueryKeyword::Update) => RlsAction::Update,
            TokenKind::Query(QueryKeyword::Delete) => RlsAction::Delete,
            TokenKind::Identifier(ref ident) if ident == "all" => RlsAction::All,
            _ => Err(self.expected("valid policy action"))?
        };
        self.advance();

        let condition = self.expression()?;
        let policy = Box::new(RlsPolicy::new(&name, action, condition));

        Ok(CreateSDL::RlsPolicy { table, policy })
    }

    fn create_user(&mut self) -> Result<CreateSDL, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::User))?;
        let name = self.identifier_name()?;

        self.expect(TokenKind::Symbol(Symbol::Colon))?; 
        let password = self.literal_string()?;

        let is_superuser = match self.current() {
            Some(token) if token.kind == TokenKind::Identifier("superuser".to_string()) => {
                self.advance();
                true
            }
            _ => false
        };

        Ok(CreateSDL::User { name, password, is_superuser })
    }
    
    fn create_role(&mut self) -> Result<CreateSDL, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Role))?;
        let name = self.identifier_name()?;
        Ok(CreateSDL::Role { name })
    }

    fn grant(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Grant))?;
        
        if let Some(token) = self.current() {
            match token.kind {
                TokenKind::SDL(SDLKeyword::Role) => return self.grant_role(),
                _ => { }
            }
        }

        let actions = self.privilege_actions()?; 

        let object = match self.current_token("privilege object")?.kind {
            TokenKind::SDL(SDLKeyword::Database) => "database",
            TokenKind::SDL(SDLKeyword::Table) => "table",
            TokenKind::SDL(SDLKeyword::Column) => "column",
            TokenKind::SDL(SDLKeyword::Policy) => "policy",
            TokenKind::SDL(SDLKeyword::User) => "user",
            TokenKind::SDL(SDLKeyword::Role) => "role",
            _ => Err(self.expected("valid privilege object"))?,
        }.to_string();
        self.advance();

        let object_name = self.identifier_name()?;

        let table = if let Some(token) = self.current() {
            if token.kind != TokenKind::Keyword(Keyword::For) {
                Some(self.identifier_name()?)
            } else {
                None
            }
        } else {
            None
        };

        self.expect(TokenKind::Keyword(Keyword::For))?;
        let role = self.identifier_name()?;

        Ok(Node::SDL(ast::SDL::Grant(ast::GrantSDL::Action { 
            object, object_name, actions, table, to: role 
        })))
    }

    fn privilege_actions(&mut self) -> Result<Vec<String>, ParserError> {
        let mut actions = vec![];
        while let Some(token) = self.current() {
            let action = match token.kind {
                TokenKind::Query(QueryKeyword::Select) => "select",
                TokenKind::Query(QueryKeyword::Insert) => "insert",
                TokenKind::Query(QueryKeyword::Update) => "udpate",
                TokenKind::Query(QueryKeyword::Delete) => "delete",
                TokenKind::SDL(SDLKeyword::Create) => "create",
                TokenKind::SDL(SDLKeyword::Drop) => "drop",
                TokenKind::SDL(SDLKeyword::Connect) => "connect",
                TokenKind::SDL(SDLKeyword::Grant) => "grant",
                TokenKind::SDL(SDLKeyword::Alter) => "alter",
                TokenKind::SDL(SDLKeyword::Execute) => "execute",
                _ => break
            };

            actions.push(action.to_string());
            self.advance();

            if let Some(token) = self.current() {
                if token.kind == TokenKind::Symbol(Symbol::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn grant_role(&mut self) -> Result<Node, ParserError> {
        self.expect(TokenKind::SDL(SDLKeyword::Role))?;

        let role = self.identifier_name()?;
        self.expect(TokenKind::Keyword(Keyword::For))?;

        let user = self.identifier_name()?;

        Ok(Node::SDL(ast::SDL::Grant(ast::GrantSDL::Role { 
            name: role, to: user 
        })))
    }

    // fn grant_database(&mut self, actions: Vec<&str>) -> Result<Vec<Privilege>, ParserError> {
    //     self.expect(TokenKind::SDL(SDLKeyword::Database))?; 
    //
    //     let object_name = self.identifier_name()?;
    //     let mut privileges = Vec::new();
    //
    //     for action in actions {
    //         let privilege = match Privilege::from_fields("database", &object_name, action, None) {
    //             Ok(privilege) => privilege,
    //             Err(e) => Err(self.expected(format!("valie privilege: {e}")))?
    //         };
    //         privileges.push(privilege);
    //     }
    //
    //     Ok(privileges)
    // }
}
