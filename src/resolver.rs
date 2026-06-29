use std::collections::HashMap;

use crate::{expr::Expr, interpreter::Interpreter, stmt::Stmt, token::Token};

#[derive(Debug)]
struct ResolveError {
    line: Option<usize>,
    place: Option<String>,
    msg: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = match self.line {
            Some(l) => l.to_string(),
            None => "unknown".to_string(),
        };
        let place = self.place.as_deref().unwrap_or("unknown");
        write!(f, "[line {}] Error at {}: {}", line, place, self.msg)
    }
}

struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolveError>,
}

impl<'a> Resolver<'a> {
    fn new(interpreter: &mut Interpreter) -> Resolver<'_> {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(expression);
            }
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.resolve_function(params, body);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            }
            Stmt::Print { expression } => self.resolve_expr(expression),
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                self.resolve_expr(initializer);
                self.define(name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
        }
    }

    fn resolve_function(&mut self, params: &[Token], body: &[Stmt]) {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee);
                for arg in arguments {
                    self.resolve_expr(arg);
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }
            Expr::Literal { .. } => {}
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { right, .. } => self.resolve_expr(right),
            Expr::Variable { name } => {
                if let Some(false) = self.scopes.last().and_then(|scope| scope.get(&name.lexeme)) {
                    self.errors.push(ResolveError {
                        line: Some(name.line),
                        place: Some(name.lexeme.clone()),
                        msg: "Can't read local variable in its own initializer.".to_string(),
                    });
                };
                self.resolve_local(expr, name);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.lexeme.as_str()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}
