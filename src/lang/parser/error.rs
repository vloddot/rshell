#![allow(clippy::module_name_repetitions)]

use crate::lang::tokens::{Token, TokenType};
use itertools::Itertools;

#[derive(Clone, Debug)]
#[repr(i32)]
pub enum ErrorKind {
    UnexpectedToken(Token, Token, Vec<TokenType>) = 1,
    RequiredTokenNotFound(Token, Token, Vec<TokenType>) = 2,
}

impl ErrorKind {
    #[must_use]
    pub fn code(self) -> i32 {
        match self {
            Self::UnexpectedToken(_, _, _) => 1,
            Self::RequiredTokenNotFound(_, _, _) => 2,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken(unexpected_token, _, _) => {
                f.write_fmt(format_args!("unexpected {}", unexpected_token.r#type))
            }
            Self::RequiredTokenNotFound(_, _, expected_tokens) => f.write_fmt(format_args!(
                "expected {}",
                expected_tokens.iter().map(ToString::to_string).join(" or ")
            )),
        }
    }
}

pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl Error {
    #[must_use]
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            ErrorKind::UnexpectedToken(unexpected_token, after_token, expected_tokens) => {
                let location = if unexpected_token.r#type == TokenType::Eof {
                    "at end".into()
                } else {
                    format!("after {:?}", after_token.lexeme)
                };

                f.write_fmt(format_args!(
                    "{}\n\nexpected {}, not {} {}",
                    self.kind,
                    expected_tokens.iter().map(ToString::to_string).join(" or "),
                    unexpected_token.r#type,
                    location
                ))
            }
            ErrorKind::RequiredTokenNotFound(found_token, after_token, expected_tokens) => {
                let location = if let TokenType::Eof = after_token.r#type {
                    String::from("at end")
                } else {
                    format!("after {:?}", after_token.lexeme)
                };

                f.write_fmt(format_args!(
                    "{}\n\nexpected {}, not {} {}",
                    self.kind,
                    expected_tokens.iter().map(ToString::to_string).join(","),
                    found_token.r#type,
                    location
                ))
            }
        }
    }
}
