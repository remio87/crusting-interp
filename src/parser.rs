use crate::expr::Expr;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;

#[derive(Debug)]
struct ParseError {
    msg: String,
}

pub type ParseResult = Result<Expr, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Expr {
        self.expression().expect("Failed to parse!!")
    }

    fn expression(&mut self) -> ParseResult {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult {
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

    fn comparison(&mut self) -> ParseResult {
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

    fn term(&mut self) -> ParseResult {
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

    fn factor(&mut self) -> ParseResult {
        let mut expr = self.unary()?;

        while self.match_expr(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult {
        if self.match_expr(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParseResult {
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
        crate::error_with_token(token, message);
        ParseError {
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
