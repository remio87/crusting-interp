use crate::expr::Expr;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;

#[derive(Debug)]
struct EvalError {
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

struct Interpreter {}

impl Interpreter {
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
                    TokenType::Minus => {
                        Self::eval_numeric_binary(left, right, |l, r| l - r, operator)
                    }
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
                    TokenType::Slash => {
                        Self::eval_numeric_binary(left, right, |l, r| l / r, operator)
                    }
                    TokenType::Star => {
                        Self::eval_numeric_binary(left, right, |l, r| l * r, operator)
                    }
                    _ => todo!(),
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
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(op(l, r))),
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
