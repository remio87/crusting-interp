use crate::expr::Expr;
use crate::token::Token;
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
        let expr = self.comparison();
        todo!()
    }

    fn comparison(&mut self) -> Expr {
        todo!()
    }

    fn match_expr(&mut self, types: &[TokenType]) -> bool {
        // todo: impl check and advance
        for tt in types {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }
        return false;
    }
}
