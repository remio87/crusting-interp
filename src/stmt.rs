use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block { statements: Vec<Stmt> },
    Expression { expression: Expr },
    Print { expression: Expr },
    Var { name: Token, initializer: Expr },
}
