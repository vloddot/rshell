use std::env;

use itertools::Itertools;

use crate::Command;

use super::tokens::{Token, TokenType};

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ErrorKind {
    UnexpectedToken(Token),
    RequiredTokenNotFound(TokenType),
}

impl ErrorKind {
    pub fn code(&self) -> u8 {
        match self {
            Self::UnexpectedToken(_) => 1,
            Self::RequiredTokenNotFound(_) => 2,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(token) => {
                f.write_fmt(format_args!("unexpected {}", token.r#type))
            }
            Self::RequiredTokenNotFound(token_type) => {
                f.write_fmt(format_args!("expected {}", token_type))
            }
        }
    }
}

pub struct Error {
    tokens: &'static [TokenType],
    kind: ErrorKind,
    location: Token,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl Error {
    pub fn new(tokens: &'static [TokenType], kind: ErrorKind, location: Token) -> Self {
        Self {
            tokens,
            kind,
            location,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            ErrorKind::UnexpectedToken(token) => {
                let location = if token.r#type == TokenType::Eof {
                    String::from("at end")
                } else {
                    format!(r#"after "{}""#, self.location.lexeme)
                };

                f.write_fmt(format_args!(
                    "{}\n\nexpected {}, not {} {}",
                    self.kind,
                    self.tokens.iter().map(ToString::to_string).join(", "),
                    token.r#type,
                    location
                ))
            }
            ErrorKind::RequiredTokenNotFound(token_type) => {
                let location = if token_type == TokenType::Eof {
                    String::from("at end")
                } else {
                    format!(r#"after "{}""#, self.location.lexeme)
                };

                f.write_fmt(format_args!(
                    "{}\n\nexpected {}, not {} {}",
                    self.kind,
                    self.tokens.iter().map(ToString::to_string).join(","),
                    token_type,
                    location
                ))
            }
        }
    }
}

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

    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

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
                        return Err(Error::new(
                            &[TokenType::Part],
                            ErrorKind::UnexpectedToken(self.peek().clone()),
                            self.previous().clone(),
                        ));
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
                            first_command.push(env::var(var).unwrap_or_default());
                        }
                        TokenType::LeftBrace => {
                            if !self.match_next(&TokenType::Part) {
                                return Err(Error::new(
                                    &[TokenType::Part],
                                    ErrorKind::UnexpectedToken(t),
                                    self.previous().clone(),
                                ));
                            }

                            let var = self.advance().lexeme.clone();

                            // If there is syntax like this: "echo ${HOME:-false}"
                            if self.r#match(&TokenType::ColonDash) && self.r#match(&TokenType::Part)
                            {
                                first_command.push(
                                    env::var(var)
                                        .unwrap_or_else(|_| self.previous().lexeme.clone()),
                                );
                            } else {
                                first_command.push(env::var(var).unwrap_or_default());
                            }

                            if !self.r#match(&TokenType::RightBrace) {
                                return Err(Error::new(
                                    &[TokenType::RightBrace],
                                    ErrorKind::RequiredTokenNotFound(self.peek().r#type.clone()),
                                    self.previous().clone(),
                                ));
                            }
                        }
                        _ => {
                            return Err(Error::new(
                                &[TokenType::Part, TokenType::LeftBrace],
                                ErrorKind::UnexpectedToken(t),
                                self.previous().clone(),
                            ))
                        }
                    }
                }
                token => {
                    eprintln!("{token:?} is not implemented currently.");
                    return Ok(Vec::new())
                },
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
