use crate::token::{LiteralValue, Token};

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Assign { name, value } => {
                write!(f, "{}", parenthesize(&name.lexeme, &[value.as_ref()]))
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(
                f,
                "{}",
                parenthesize(&operator.lexeme, &[left.as_ref(), right.as_ref()])
            ),
            Expr::Grouping { expression } => {
                write!(f, "{}", parenthesize("group", &[expression.as_ref()]))
            }
            Expr::Literal { value } => {
                write!(f, "{}", value)
            }
            Expr::Unary { operator, right } => {
                write!(f, "{}", parenthesize(&operator.lexeme, &[right.as_ref()]))
            }
            Expr::Variable { name } => {
                write!(f, "{}", &name.lexeme)
            }
        }
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    format!(
        "({} {})",
        name,
        exprs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{LiteralValue, Token};
    use crate::token_type::TokenType;

    #[test]
    fn test_pretty_print() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(123.0),
                }),
            }),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
                line: 1,
            },
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: LiteralValue::Number(45.67),
                }),
            }),
        };

        assert_eq!(expr.to_string(), "(* (- 123) (group 45.67))");
    }
}
