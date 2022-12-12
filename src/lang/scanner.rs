use crate::lang::tokens::{Token, TokenType};

#[derive(Clone)]
pub struct Scanner {
    start: usize,
    current: usize,
    tokens: Vec<Token>,
    source: String,
}

impl Scanner {
    fn add_token(&mut self, r#type: TokenType) {
        let text: String = self.source.chars().collect::<Vec<_>>()[self.start..self.current]
            .iter()
            .collect();

        self.tokens.push(Token::new(r#type, text, self.current));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().collect::<Vec<_>>()[(self.current - 1) as usize]
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_alphanumeric() || vec!['=', '\'', '"', '.', '/'].contains(&c)
    }

    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    #[must_use]
    pub fn new(source: String) -> Self {
        Self {
            start: 0,
            current: 0,
            tokens: Vec::new(),
            source,
        }
    }

    fn part(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        self.add_token(TokenType::Part);
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().collect::<Vec<_>>()[self.current]
        }
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.chars().collect::<Vec<_>>()[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '&' => {
                if self.r#match('&') {
                    self.add_token(TokenType::AndAnd);
                } else {
                    self.add_token(TokenType::And);
                }
            }
            '|' => {
                if self.r#match('|') {
                    self.add_token(TokenType::OrOr);
                } else {
                    self.add_token(TokenType::Pipe);
                }
            }
            '$' => self.add_token(TokenType::DollarSign),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ' ' | '\n' | '\t' | '\r' => {}
            ':' => {
                if self.r#match('-') {
                    self.add_token(TokenType::ColonDash);
                }
            }
            ';' => self.add_token(TokenType::Semicolon),
            _ => self.part(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        // EOF
        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.current));

        self.tokens.clone()
    }
}
