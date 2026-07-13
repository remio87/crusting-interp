use std::rc::Rc;

use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "Parse error: {}", self.msg)
        match self.token.token_type {
            TokenType::Eof => {
                write!(f, "[line {}] Error at end: {}", self.token.line, self.msg)
            }
            _ => {
                write!(
                    f,
                    "[line {}] Error at {}: {}",
                    self.token.line, self.token.lexeme, self.msg
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;
pub type ParseResultThrough = Result<Vec<Stmt>, Vec<ParseError>>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // Errors which do not into panic mode
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> ParseResultThrough {
        let mut statements = Vec::<Stmt>::new();
        let mut errors = Vec::<ParseError>::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => {
                    statements.push(stmt);
                    errors.append(&mut self.errors)
                }
                Err(e) => {
                    self.synchronize();
                    errors.append(&mut self.errors);
                    errors.push(e)
                }
            }
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_expr(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.match_expr(&[TokenType::Fun]) {
            self.function("function")
        } else if self.match_expr(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self
            .consume(TokenType::Identifier, "Expect class name")?
            .clone();

        let superclass = if self.match_expr(&[TokenType::Less]) {
            self.consume(TokenType::Identifier, "Expect superclass name")?;
            Some(Expr::Variable {
                name: self.previous().clone(),
            })
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            superclass,
            methods,
        })
    }

    fn function(&mut self, kind: &str) -> ParseResult<Stmt> {
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {} name.", kind))?
            .clone();

        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut params: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    self.errors.push(ParseError {
                        token: self.peek().clone(),
                        msg: "Can't have more than 255 parameters.".to_string(),
                    });
                }
                params.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.")?
                        .clone(),
                );
                if !self.match_expr(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block_statements()?;

        Ok(Stmt::Function {
            name,
            params,
            body: Rc::new(body),
        })
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_expr(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_expr(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_expr(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_expr(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_expr(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_expr(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();
        let initializer = if self.match_expr(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal {
                value: LiteralValue::Nil,
            }
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_expr(&[TokenType::Semicolon]) {
            None
        } else if self.match_expr(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment,
                    },
                ],
            };
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal {
                value: LiteralValue::Bool(true),
            }),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_expr(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self.previous().clone();
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn block_statements(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::<Stmt>::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn block(&mut self) -> ParseResult<Stmt> {
        Ok(Stmt::Block {
            statements: self.block_statements()?,
        })
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression })
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.match_expr(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Get { object, name } => {
                    return Ok(Expr::Set {
                        object,
                        name,
                        value: Box::new(value),
                    });
                }
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                _ => {
                    self.errors.push(ParseError {
                        token: equals,
                        msg: "Invalid assignment target.".to_string(),
                    });
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.match_expr(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_expr(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_expr(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_expr(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_expr(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_expr(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_expr(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.match_expr(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_expr(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name: name.clone(),
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.errors.push(ParseError {
                        token: self.peek().clone(),
                        msg: "Can't have more than 255 arguments.".to_string(),
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_expr(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren: paren.clone(),
            arguments,
        })
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        match self.peek().token_type {
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Bool(false),
                })
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Bool(true),
                })
            }
            TokenType::Nil => {
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::Nil,
                })
            }
            TokenType::Number | TokenType::String => {
                self.advance();
                Ok(Expr::Literal {
                    value: self
                        .previous()
                        .literal
                        .clone()
                        .expect("Literal value must be Some"),
                })
            }
            TokenType::Super => {
                self.advance();
                let keyword = self.previous().clone();
                self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
                let method = self
                    .consume(TokenType::Identifier, "Expect superclass method name.")?
                    .clone();
                Ok(Expr::Super { keyword, method })
            }
            TokenType::This => {
                self.advance();
                Ok(Expr::This {
                    keyword: self.previous().clone(),
                })
            }
            TokenType::Identifier => Ok(Expr::Variable {
                name: self.advance().clone(),
            }),
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression")
                    .map(|_| Expr::Grouping {
                        expression: Box::new(expr),
                    })
            }
            _ => Err(Self::error(self.peek(), "Expect expression.")),
        }
    }

    fn match_expr(&mut self, types: &[TokenType]) -> bool {
        for tt in types {
            if self.check(*tt) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(Self::error(self.peek(), message))
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(token: &Token, message: &str) -> ParseError {
        ParseError {
            token: token.clone(),
            msg: message.to_string(),
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class => return,
                TokenType::Fun => return,
                TokenType::Var => return,
                TokenType::For => return,
                TokenType::If => return,
                TokenType::While => return,
                TokenType::Print => return,
                TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
