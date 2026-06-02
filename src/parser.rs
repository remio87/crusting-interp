use crate::expr::Expr;
use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_expr(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_expr(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_expr(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_expr(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_expr(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();
            Expr::Unary {
                operator,
                right: Box::new(right),
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        match self.peek().token_type {
            TokenType::False => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Bool(false),
                }
            }
            TokenType::True => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Bool(true),
                }
            }
            TokenType::Nil => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Nil,
                }
            }
            TokenType::Number | TokenType::String => {
                self.advance();
                Expr::Literal {
                    value: self.previous().literal.expect("Literal value must be Some"),
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Expect ')' after expression");
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            _ => panic!("Unexpected token"),
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
}
