use std::collections::HashMap;

use crate::{expr::Expr, interpreter::Interpreter, stmt::Stmt, token::Token};

#[derive(Debug)]
pub struct ResolveError {
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
    errors: Vec<ResolveError>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver<'_> {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            errors: Vec::new(),
        }
    }

    pub fn resolve(mut self, stmts: &[Stmt]) -> Result<(), Vec<ResolveError>> {
        self.resolve_stmts(stmts);
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
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
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;
                self.declare(name);
                self.define(name);

                if let Some(superclass) = superclass {
                    match superclass {
                        Expr::Variable { name: sc_name } => {
                            if name.lexeme == sc_name.lexeme {
                                self.errors.push(ResolveError {
                                    line: Some(sc_name.line),
                                    place: Some(sc_name.lexeme.clone()),
                                    msg: "A class can't inherit from itself.".to_string(),
                                });
                            }
                            self.resolve_expr(superclass);
                        }
                        _ => unreachable!(),
                    }
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_string(), true);

                for method in methods {
                    match method {
                        Stmt::Function { name, params, body } => {
                            self.resolve_function(
                                params,
                                body,
                                if name.lexeme == "init" {
                                    FunctionType::Initializer
                                } else {
                                    FunctionType::Method
                                },
                            );
                        }
                        _ => self.errors.push(ResolveError {
                            line: Some(name.line),
                            place: Some(name.lexeme.clone()),
                            msg: "Methods need to be function statement.".to_string(),
                        }),
                    }
                }

                self.end_scope();
                self.current_class = enclosing_class;
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(expression);
            }
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.resolve_function(params, body, FunctionType::Function);
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
            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::None {
                    self.errors.push(ResolveError {
                        line: Some(keyword.line),
                        place: Some(keyword.lexeme.clone()),
                        msg: "Can't return from top-level code.".to_string(),
                    });
                }

                if let Some(value) = value {
                    if self.current_function == FunctionType::Initializer {
                        self.errors.push(ResolveError {
                            line: Some(keyword.line),
                            place: Some(keyword.lexeme.clone()),
                            msg: "Can't return value from an initializer".to_string(),
                        });
                    }
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

    fn resolve_function(&mut self, params: &[Token], body: &[Stmt], fn_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = fn_type;

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();

        self.current_function = enclosing_function;
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
            Expr::Get { object, .. } => {
                self.resolve_expr(object);
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }
            Expr::Literal { .. } => {}
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            }
            Expr::This { keyword } => {
                if self.current_class == ClassType::None {
                    self.errors.push(ResolveError {
                        line: Some(keyword.line),
                        place: Some(keyword.lexeme.clone()),
                        msg: "Can't use 'this' outside a class.".to_string(),
                    })
                } else {
                    self.resolve_local(expr, keyword);
                }
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
            if scope.contains_key(&name.lexeme) {
                self.errors.push(ResolveError {
                    line: Some(name.line),
                    place: Some(name.lexeme.clone()),
                    msg: "Already a variable with this name in this scope.".to_string(),
                });
            }
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
