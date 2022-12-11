use std::env;

use crate::Command;

use super::tokens::{Token, TokenType};

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ErrorKind {
    UnexpectedToken(Token),
}

impl ErrorKind {
    pub fn code(&self) -> u8 {
        match self {
            Self::UnexpectedToken(_) => 1,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(token) => {
                let lexeme = if token.r#type == TokenType::Eof {
                    "<eof>"
                } else {
                    token.lexeme.as_str()
                };

                f.write_fmt(format_args!("unexpected {:?} token", lexeme))
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
        match self.kind.clone() {
            ErrorKind::UnexpectedToken(token) => {
                let lexeme = if token.r#type == TokenType::Eof {
                    "<eof>"
                } else {
                    token.lexeme.as_str()
                };

                f.write_fmt(format_args!(
                    "{}; expected {:?}, not {:?} after {:?}",
                    self.kind, self.tokens, lexeme, self.location.lexeme
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
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn check(&self, r#type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().r#type == r#type
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::Eof
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Command>, Error> {
        let mut commands = Vec::new();
        let mut first_command = Vec::new();

        loop {
            let t = self.peek().clone();
            self.advance();
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

                    let other_commands = self.parse()?;

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
                    if self.r#match(&TokenType::Part) {
                        let previous = self.previous().lexeme.clone();
                        first_command.push(env::var(previous).unwrap_or_default());
                    } else {
                        return Err(Error::new(
                            &[TokenType::Part],
                            ErrorKind::UnexpectedToken(self.peek().clone()),
                            self.previous().clone(),
                        ));
                    }
                }
                TokenType::Pipe => unimplemented!(),
                TokenType::OrOr => unimplemented!(),
                TokenType::Semicolon => unimplemented!(),
                TokenType::LeftBrace => unimplemented!(),
                TokenType::RightBrace => unimplemented!(),
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
