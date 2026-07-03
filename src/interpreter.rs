use crate::class::Class;
use crate::environment::Environment;
use crate::expr::Expr;
use crate::instance::Instance;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct ReturnValue {
    keyword: Token,
    value: Value,
}

#[derive(Debug)]
pub enum EvalError {
    RuntimeError {
        line: Option<usize>,
        place: Option<String>,
        msg: String,
    },
    Return(Box<ReturnValue>),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::RuntimeError { line, place, msg } => {
                let line = match line {
                    Some(l) => format!("{}", l),
                    None => "unknown".to_string(),
                };
                let place = place.clone().unwrap_or("unknown".to_string());
                write!(f, "[line {}] Error at {}: {}", line, place, msg)
            }
            EvalError::Return(ret) => {
                write!(f, "Value {} returned from {}", ret.value, ret.keyword)
            }
        }
    }
}

impl std::error::Error for EvalError {}

type EvalResult = Result<Value, EvalError>;

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<*const Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new(None);
        globals.define(
            "clock".to_string(),
            Value::NativeFunction {
                name: "clock".to_string(),
                arity: 0,
                function: Rc::new(|_| {
                    Value::Number(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                    )
                }),
            },
        );
        let globals = Rc::new(RefCell::new(globals));
        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::clone(&globals),
            locals: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr as *const Expr, depth);
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> EvalResult {
        for statement in statements {
            match self.execute(&statement) {
                Err(EvalError::Return(ret)) => {
                    return Err(EvalError::RuntimeError {
                        line: Some(ret.keyword.line),
                        place: Some(ret.keyword.lexeme.clone()),
                        msg: "Can't return from top-level code.".to_string(),
                    });
                }
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }
        Ok(Value::Nil)
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> EvalResult {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;
        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement).map(|_| ()))
            .map(|_| Value::Nil);
        self.environment = previous;
        result
    }

    fn execute(&mut self, statement: &Stmt) -> EvalResult {
        match statement {
            Stmt::Block { statements } => {
                let environment = Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                    &self.environment,
                )))));
                self.execute_block(statements, environment)
            }
            Stmt::Class { name, .. } => {
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_string(), Value::Nil);
                let class = Value::LoxClass(Rc::new(Class::new(name.lexeme.as_ref())));
                self.environment
                    .borrow_mut()
                    .assign(name.lexeme.to_string(), class)
                    .unwrap();
                Ok(Value::Nil)
            }
            Stmt::Expression { expression } => {
                self.eval(expression)?;
                Ok(Value::Nil)
            }
            Stmt::Function { name, params, body } => {
                self.environment.borrow_mut().define(
                    name.lexeme.clone(),
                    Value::LoxFunction {
                        name: name.lexeme.clone(),
                        args: params.clone(),
                        closure: Rc::clone(&self.environment),
                        body: Rc::clone(body),
                    },
                );
                Ok(Value::Nil)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if Self::is_truthy(&self.eval(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(stmt) = else_branch {
                    self.execute(stmt)?;
                }
                Ok(Value::Nil)
            }
            Stmt::Print { expression } => {
                println!("{}", self.eval(expression)?);
                Ok(Value::Nil)
            }
            Stmt::Return { keyword, value } => {
                let value = match value {
                    Some(expr) => self.eval(expr)?,
                    None => Value::Nil,
                };
                let keyword = keyword.clone();
                let ret = Box::new(ReturnValue { keyword, value });
                Err(EvalError::Return(ret))
            }
            Stmt::Var { name, initializer } => {
                let val = self.eval(initializer)?;
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), val);
                Ok(Value::Nil)
            }
            Stmt::While { condition, body } => {
                while Self::is_truthy(&self.eval(condition)?) {
                    self.execute(body)?;
                }
                Ok(Value::Nil)
            }
        }
    }

    fn eval(&mut self, expr: &Expr) -> EvalResult {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.eval(value)?;
                match self.locals.get(&(expr as *const Expr)) {
                    Some(&distance) => {
                        match Environment::assign_at(
                            Rc::clone(&self.environment),
                            distance,
                            name.lexeme.clone(),
                            value.clone(),
                        ) {
                            Ok(_) => Ok(value),
                            Err(e) => Err(EvalError::RuntimeError {
                                line: Some(name.line),
                                place: Some(name.lexeme.clone()),
                                msg: e,
                            }),
                        }
                    }
                    None => {
                        match self
                            .environment
                            .borrow_mut()
                            .assign(name.lexeme.clone(), value.clone())
                        {
                            Ok(()) => Ok(value),
                            Err(e) => Err(EvalError::RuntimeError {
                                line: Some(name.line),
                                place: Some(name.lexeme.clone()),
                                msg: e,
                            }),
                        }
                    }
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;
                match operator.token_type {
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    TokenType::Greater => {
                        Self::eval_numeric_binary(left, right, |l, r| Value::Bool(l > r), operator)
                    }
                    TokenType::GreaterEqual => {
                        Self::eval_numeric_binary(left, right, |l, r| Value::Bool(l >= r), operator)
                    }
                    TokenType::Less => {
                        Self::eval_numeric_binary(left, right, |l, r| Value::Bool(l < r), operator)
                    }
                    TokenType::LessEqual => {
                        Self::eval_numeric_binary(left, right, |l, r| Value::Bool(l <= r), operator)
                    }
                    TokenType::Minus => Self::eval_numeric_binary(
                        left,
                        right,
                        |l, r| Value::Number(l - r),
                        operator,
                    ),
                    TokenType::Plus => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::Str(l), Value::Str(r)) => Ok(Value::Str(l + &r)),
                        _ => Err(EvalError::RuntimeError {
                            line: Some(operator.line),
                            place: Some(operator.lexeme.clone()),
                            msg: "Not both of left and right are either number or string."
                                .to_string(),
                        }),
                    },
                    TokenType::Slash => Self::eval_numeric_binary(
                        left,
                        right,
                        |l, r| Value::Number(l / r),
                        operator,
                    ),
                    TokenType::Star => Self::eval_numeric_binary(
                        left,
                        right,
                        |l, r| Value::Number(l * r),
                        operator,
                    ),
                    _ => unreachable!("All the binary operators must be covered."),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| self.eval(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee {
                    Value::LoxClass(class) => Ok(Value::LoxInstance(Rc::new(Instance::new(class)))),
                    Value::LoxFunction {
                        args,
                        closure,
                        body,
                        ..
                    } => {
                        if arguments.len() != args.len() {
                            return Err(EvalError::RuntimeError {
                                line: Some(paren.line),
                                place: Some(paren.lexeme.clone()),
                                msg: format!(
                                    "Expected {} arguments, but got {}.",
                                    args.len(),
                                    arguments.len()
                                ),
                            });
                        }
                        let mut environment = Environment::new(Some(closure));
                        args.iter()
                            .zip(arguments.iter())
                            .for_each(|(token, value)| {
                                environment.define(token.lexeme.clone(), value.clone());
                            });
                        match self.execute_block(&body, Rc::new(RefCell::new(environment))) {
                            Err(EvalError::Return(ret)) => Ok(ret.value),
                            result => result,
                        }
                    }
                    Value::NativeFunction {
                        arity, function, ..
                    } => {
                        if arguments.len() != arity {
                            return Err(EvalError::RuntimeError {
                                line: Some(paren.line),
                                place: Some(paren.lexeme.clone()),
                                msg: format!(
                                    "Expected {} arguments, but got {}.",
                                    arity,
                                    arguments.len()
                                ),
                            });
                        }
                        Ok(function(arguments))
                    }
                    _ => Err(EvalError::RuntimeError {
                        line: Some(paren.line),
                        place: Some(paren.lexeme.clone()),
                        msg: "Can only call functions and classes.".to_string(),
                    }),
                }
            }
            Expr::Grouping { expression } => self.eval(expression),
            Expr::Literal { value } => Ok(value.clone().into()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.eval(left)?;
                match operator.token_type {
                    TokenType::Or => {
                        if Self::is_truthy(&left) {
                            Ok(left)
                        } else {
                            self.eval(right)
                        }
                    }
                    TokenType::And => {
                        if !Self::is_truthy(&left) {
                            Ok(left)
                        } else {
                            self.eval(right)
                        }
                    }
                    _ => unreachable!("Invalid logical operator."),
                }
            }
            Expr::Unary { operator, right } => {
                let right = self.eval(right)?;
                match operator.token_type {
                    TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(&right))),
                    TokenType::Minus => match right {
                        Value::Number(num) => Ok(Value::Number(-num)),
                        _ => Err(EvalError::RuntimeError {
                            line: Some(operator.line),
                            place: Some(operator.lexeme.clone()),
                            msg: "Unary minus can only be applied to number.".to_string(),
                        }),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Variable { name } => self.look_up_variable(name, expr),
        }
    }

    fn look_up_variable(&mut self, name: &Token, expr: &Expr) -> EvalResult {
        if let Some(&distance) = self.locals.get(&(expr as *const Expr)) {
            match Environment::get_at(Rc::clone(&self.environment), distance, &name.lexeme) {
                Ok(v) => Ok(v),
                Err(e) => Err(EvalError::RuntimeError {
                    line: Some(name.line),
                    place: Some(name.lexeme.clone()),
                    msg: e,
                }),
            }
        } else {
            match self.globals.borrow().get(name.lexeme.as_str()) {
                Ok(v) => Ok(v),
                Err(e) => Err(EvalError::RuntimeError {
                    line: Some(name.line),
                    place: Some(name.lexeme.clone()),
                    msg: e,
                }),
            }
        }
    }

    fn eval_numeric_binary<F>(left: Value, right: Value, op: F, token: &Token) -> EvalResult
    where
        F: Fn(f64, f64) -> Value,
    {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(op(l, r)),
            _ => Err(EvalError::RuntimeError {
                line: Some(token.line),
                place: Some(token.lexeme.clone()),
                msg: "Not both of left and right are number.".to_string(),
            }),
        }
    }

    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Number(_) => true,
            Value::Str(_) => true,
            Value::Bool(b) => *b,
            Value::LoxFunction { .. } => true,
            Value::NativeFunction { .. } => true,
            Value::LoxClass(_) => true,
            Value::LoxInstance(_) => true,
            Value::Nil => false,
        }
    }
}
