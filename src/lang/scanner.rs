use crate::{
    lang::tokens::{Token, TokenType},
    ALIASES, PREVIOUS_EXIT_CODE,
};

#[derive(Clone)]
pub(crate) struct Scanner {
    start: usize,
    current: usize,
    tokens: Vec<Token>,
    source: Vec<char>,
}

impl Scanner {
    fn add_token(&mut self, r#type: TokenType) {
        let text: String = self.source[self.start..self.current].iter().collect();

        self.tokens.push(Token::new(r#type, text, self.current));
    }

    fn add_token_with_lexeme(&mut self, r#type: TokenType, lexeme: String) {
        self.tokens.push(Token::new(r#type, lexeme, self.current));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_part(c: char) -> bool {
        c.is_alphanumeric() || vec!['=', '\'', '"', '.', '/', '-'].contains(&c)
    }

    #[must_use]
    pub(crate) fn new(source: &str) -> Self {
        Self {
            start: 0,
            current: 0,
            tokens: Vec::new(),
            source: source.chars().collect::<Vec<_>>(),
        }
    }

    async fn part(&mut self, start: usize) {
        let mut inside_quotes = false;
        let mut c = self.peek();

        while Self::is_part(c) || (inside_quotes && c == ' ') {
            self.advance();
            c = self.peek();

            inside_quotes = if vec!['\'', '"'].contains(&c) || inside_quotes {
                true
            } else {
                vec!['\'', '"'].contains(&c) && !inside_quotes
            };
        }

        let text: String = self.source[start..self.current].iter().collect();

        let alias_lock = ALIASES.lock().await;

        if let Some(value) = alias_lock.get(text.as_str()) {
            self.add_token_with_lexeme(TokenType::Part, value.to_string());
            return;
        }

        self.add_token(TokenType::Part);
    }

    async fn part_return_lexeme(&mut self, start: usize) -> String {
        let mut inside_quotes = false;
        let mut c = self.peek();

        while Self::is_part(c) || (inside_quotes && c == ' ') {
            self.advance();
            c = self.peek();

            inside_quotes = if vec!['\'', '"'].contains(&c) || inside_quotes {
                true
            } else {
                vec!['\'', '"'].contains(&c) && !inside_quotes
            };
        }

        let text: String = self.source[start..self.current].iter().collect();

        let alias_lock = ALIASES.lock().await;

        if let Some(value) = alias_lock.get(text.as_str()) {
            value.to_string()
        } else {
            text
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn r#match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    async fn scan_token(&mut self) {
        match self.advance() {
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
            '$' => {
                if self.r#match('?') {
                    let previous_exit_code = *PREVIOUS_EXIT_CODE.lock().await;
                    self.add_token_with_lexeme(TokenType::Part, previous_exit_code.to_string());
                    return;
                }
                self.add_token(TokenType::DollarSign);
            }
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ' ' | '\n' | '\t' | '\r' => {}
            ':' => {
                if self.r#match('-') {
                    self.add_token(TokenType::ColonDash);
                }
            }
            '~' => {
                let text = format!(
                    "{}{}",
                    std::env::var("HOME").unwrap_or_default(),
                    if Self::is_part(self.advance()) {
                        self.part_return_lexeme(self.start + 1).await
                    } else {
                        String::new()
                    }
                );

                self.add_token_with_lexeme(TokenType::Part, text);
            }
            ';' => self.add_token(TokenType::Semicolon),
            _ => self.part(self.start).await,
        }
    }

    pub(crate) async fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token().await;
        }

        // EOF
        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.current));

        self.tokens.clone()
    }
}
