use crate::environment::Environment;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;

#[derive(Debug)]
pub struct EvalError {
    line: Option<usize>,
    place: Option<String>,
    msg: String,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = match self.line {
            Some(l) => format!("{}", l),
            None => "unknown".to_string(),
        };
        let place = self.place.clone().unwrap_or("unknown".to_string());
        write!(f, "[line {}] Error at {}: {}", line, place, self.msg)
    }
}

impl std::error::Error for EvalError {}

type EvalResult = Result<Value, EvalError>;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> EvalResult {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(Value::Nil)
    }

    fn execute(&mut self, statement: Stmt) -> EvalResult {
        match statement {
            Stmt::Expression { expression } => {
                self.eval(expression)?;
                Ok(Value::Nil)
            }
            Stmt::Print { expression } => {
                println!("{}", self.eval(expression)?);
                Ok(Value::Nil)
            }
            Stmt::Var { name, initializer } => {
                let val = self.eval(initializer)?;
                self.environment.define(name.lexeme, val);
                Ok(Value::Nil)
            }
        }
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.eval(*value)?;
                match self.environment.assign(name.lexeme.clone(), value.clone()) {
                    Ok(()) => Ok(value),
                    Err(e) => Err(EvalError {
                        line: Some(name.line),
                        place: Some(name.lexeme),
                        msg: e,
                    }),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.eval(*left)?;
                let right = self.eval(*right)?;
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
                        _ => Err(EvalError {
                            line: Some(operator.line),
                            place: Some(operator.lexeme),
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
            Expr::Grouping { expression } => self.eval(*expression),
            Expr::Literal { value } => Ok(value.into()),
            Expr::Unary { operator, right } => {
                let right = self.eval(*right)?;
                match operator.token_type {
                    TokenType::Bang => Ok(Value::Bool(!Self::is_truthy(right))),
                    TokenType::Minus => match right {
                        Value::Number(num) => Ok(Value::Number(-num)),
                        _ => Err(EvalError {
                            line: Some(operator.line),
                            place: Some(operator.lexeme),
                            msg: "Unary minus can only be applied to number.".to_string(),
                        }),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Variable { name } => match self.environment.get(&name.lexeme) {
                Ok(v) => Ok(v.to_owned()),
                Err(e) => Err(EvalError {
                    line: Some(name.line),
                    place: Some(name.lexeme),
                    msg: e,
                }),
            },
        }
    }

    fn eval_numeric_binary<F>(left: Value, right: Value, op: F, token: Token) -> EvalResult
    where
        F: Fn(f64, f64) -> Value,
    {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(op(l, r)),
            _ => Err(EvalError {
                line: Some(token.line),
                place: Some(token.lexeme),
                msg: "Not both of left and right are number.".to_string(),
            }),
        }
    }

    fn is_truthy(val: Value) -> bool {
        match val {
            Value::Number(_) => true,
            Value::Str(_) => true,
            Value::Bool(b) => b,
            Value::Nil => false,
        }
    }
}
