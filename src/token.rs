use crate::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: usize,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} ", self.token_type, self.lexeme)?;
        match &self.literal {
            Some(literal) => write!(f, "{:?}", literal),
            None => write!(f, "{}", "null"),
        }
    }
}
