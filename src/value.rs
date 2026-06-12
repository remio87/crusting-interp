use crate::token::LiteralValue;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => {
                let s = format!("{}", num);
                let s = if s.contains('.') {
                    s.trim_end_matches('0').trim_end_matches('.')
                } else {
                    &s
                };
                write!(f, "{}", s)
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl From<LiteralValue> for Value {
    fn from(value: LiteralValue) -> Self {
        match value {
            LiteralValue::Number(num) => Self::Number(num),
            LiteralValue::Str(s) => Self::Str(s),
            LiteralValue::Bool(b) => Self::Bool(b),
            LiteralValue::Nil => Self::Nil,
        }
    }
}
