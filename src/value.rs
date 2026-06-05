use crate::token::LiteralValue;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
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
