use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;
use std::collections::HashMap;
use std::sync::LazyLock;

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    let keywords = [
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ];
    keywords.into_iter().collect()
});

#[derive(Debug)]
struct ScanError {
    line: usize,
    msg: String,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.msg)
    }
}

#[derive(Debug)]
pub struct ScanErrors(Vec<ScanError>);

impl std::fmt::Display for ScanErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl std::error::Error for ScanErrors {}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    errors: Vec<ScanError>,
}

type ScanResult = Result<Vec<Token>, ScanErrors>;

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            errors: Vec::new(),
        }
    }

    pub fn into_tokens(self) -> ScanResult {
        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(ScanErrors(self.errors))
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('!') => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            Some('=') => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            Some('<') => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            Some('>') => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            Some('/') => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            Some(' ') => {}
            Some('\r') => {}
            Some('\t') => {}
            Some('\n') => self.line += 1,
            Some('"') => self.string(),
            Some(c) if Scanner::is_digit(c) => self.number(),
            Some(c) if Scanner::is_alpha(c) => self.identifier(),
            Some(c) => self.error(self.line, &format!("Unexpected character: {}", c)),
            None => unreachable!("No more characters to scan"),
        }
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }
        // handle fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume "."
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }
        let num = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .expect("failed to parse numeric literal");
        self.add_token_with_literal(TokenType::Number, Some(LiteralValue::Number(num)));
    }

    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = KEYWORDS.get(text.as_str()).copied();
        match token_type {
            Some(tt) => self.add_token(tt),
            None => self.add_token(TokenType::Identifier),
        }
    }

    fn string(&mut self) {
        // find closing double quotation position
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        // no closing found
        if self.is_at_end() {
            self.error(self.line, "Unterminated string");
            return;
        }
        // the closing "
        self.advance();
        // add token with literal value. note that quotations are excluded.
        let value: String = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect();
        self.add_token_with_literal(TokenType::String, Some(LiteralValue::Str(value)));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if expected == self.source[self.current] {
            self.current += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || (c == '_')
    }

    fn is_alpha_numeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.get(self.current).copied();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None)
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line: self.line,
        });
    }

    fn error(&mut self, line: usize, msg: &str) {
        self.errors.push(ScanError {
            line,
            msg: msg.to_string(),
        });
    }
}
