use crate::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
enum LiteralValue {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<LiteralValue>,
    line: i32,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {}",
            self.token_type,
            self.lexeme,
            match &self.literal {
                Some(literal) => format!("{:?}", literal),
                None => "null".to_string(),
            }
        )
    }
}
