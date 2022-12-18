use super::tokens::{Token, TokenType};
use crate::Command;
use error::{Error, ErrorKind};

pub mod error;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().r#type == r#type
        }
    }

    fn check_next(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek_next().r#type == r#type
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::Eof
    }

    fn match_next(&mut self, r#type: &TokenType) -> bool {
        if self.check_next(r#type) {
            self.advance();
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Returns the parse tokens of this [`Parser`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn parse_tokens(&mut self) -> Result<Vec<Command>, Error> {
        let mut commands = Vec::new();
        let mut first_command = Vec::new();

        // EOF token
        if self.is_at_end() {
            return Ok(Vec::new());
        }

        while !self.is_at_end() {
            let t = self.advance().clone();
            match t.r#type {
                TokenType::AndAnd => {
                    let next_token = self.peek();

                    if vec![
                        TokenType::Pipe,
                        TokenType::And,
                        TokenType::AndAnd,
                        TokenType::Eof,
                        TokenType::OrOr,
                        TokenType::Semicolon,
                    ]
                    .contains(&next_token.r#type)
                    {
                        return Err(Error::new(ErrorKind::UnexpectedToken(
                            next_token.clone(),
                            t,
                            vec![TokenType::DollarSign, TokenType::Part],
                        )));
                    }

                    let other_commands = self.parse_tokens()?;

                    for command in other_commands {
                        commands.push(command);
                    }
                }

                TokenType::And => unimplemented!(),

                TokenType::Part => {
                    first_command.push(t.lexeme);
                }

                // end of command
                TokenType::Eof => break,

                TokenType::DollarSign => {
                    let t = self.peek().clone();
                    match t.r#type {
                        TokenType::Part => {
                            let var = self.advance().lexeme.clone();
                            first_command.push(std::env::var(var).unwrap_or_default());
                        }
                        TokenType::LeftBrace => {
                            if !self.match_next(&TokenType::Part) {
                                return Err(Error::new(ErrorKind::UnexpectedToken(
                                    self.peek_next().clone(),
                                    t,
                                    vec![TokenType::Part],
                                )));
                            }

                            let var = self.advance().lexeme.clone();

                            // If there is syntax like this: "echo ${HOME:-false}"
                            if self.r#match(&TokenType::ColonDash) && self.r#match(&TokenType::Part)
                            {
                                first_command.push(
                                    std::env::var(var)
                                        .unwrap_or_else(|_| self.previous().lexeme.clone()),
                                );
                            } else {
                                first_command.push(std::env::var(var).unwrap_or_default());
                            }

                            if !self.r#match(&TokenType::RightBrace) {
                                return Err(Error::new(ErrorKind::RequiredTokenNotFound(
                                    self.peek().clone(),
                                    self.peek_back().clone(),
                                    vec![TokenType::RightBrace],
                                )));
                            }
                        }
                        _ => {
                            return Err(Error::new(ErrorKind::UnexpectedToken(
                                t,
                                self.peek_back().clone(),
                                vec![TokenType::Part, TokenType::LeftBrace],
                            )))
                        }
                    }
                }
                token => {
                    eprintln!("{token:?} is not implemented currently.");
                    return Ok(Vec::new());
                }
            }
        }

        commands.insert(
            0,
            Command::new(first_command[0].clone(), first_command[1..].to_vec()),
        );

        Ok(commands)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_back(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek_next(&self) -> &Token {
        &self.tokens[self.current + 1]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn r#match(&mut self, r#type: &TokenType) -> bool {
        if self.check(r#type) {
            self.advance();
            true
        } else {
            false
        }
    }
}
