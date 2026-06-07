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

pub struct Interpreter {}

impl Interpreter {
    pub fn interpret(statements: Vec<Stmt>) -> EvalResult {
        for statement in statements {
            Self::execute(statement)?;
        }
        Ok(Value::Nil)
    }

    fn execute(statement: Stmt) -> EvalResult {
        match statement {
            Stmt::Expression { expression } => {
                Self::eval(expression)?;
                Ok(Value::Nil)
            }
            Stmt::Print { expression } => {
                println!("{}", Self::eval(expression)?);
                Ok(Value::Nil)
            }
        }
    }

    fn eval(expr: Expr) -> EvalResult {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = Self::eval(*left)?;
                let right = Self::eval(*right)?;
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
            Expr::Grouping { expression } => Self::eval(*expression),
            Expr::Literal { value } => Ok(value.into()),
            Expr::Unary { operator, right } => {
                let right = Self::eval(*right)?;
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
