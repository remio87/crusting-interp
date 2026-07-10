use std::rc::Rc;

use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Rc<Vec<Stmt>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
