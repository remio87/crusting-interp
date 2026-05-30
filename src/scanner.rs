use crate::token::{LiteralValue, Token};
use crate::token_type::TokenType;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
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
            Some(c) => crate::error(self.line, &format!("Unexpected character: {}", c)),
            None => unreachable!("No more characters to scan"),
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
            crate::error(self.line, "Unterminated string");
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
}
