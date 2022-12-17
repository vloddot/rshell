use itertools::Itertools;
use crate::lang::tokens::TokenType;
use crate::lang::tokens::Token;

#[derive(Clone, Debug)]
#[repr(i32)]
pub enum Kind {
    UnexpectedToken(Token, Token, Vec<TokenType>) = 1,
    RequiredTokenNotFound(Token, Token, Vec<TokenType>) = 2,
}

impl Kind {
    #[must_use]
    pub fn code(self) -> i32 {
        match self {
            Self::UnexpectedToken(_, _, _) => 1,
            Self::RequiredTokenNotFound(_, _, _) => 2,
        }
    }
}

impl std::fmt::Display for Kind {
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
    kind: Kind,
}

impl Error {
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind.clone()
    }
}

impl Error {
    #[must_use]
    pub fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            Kind::UnexpectedToken(unexpected_token, after_token, expected_tokens) => {
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
            Kind::RequiredTokenNotFound(found_token, after_token, expected_tokens) => {
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
