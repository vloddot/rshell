use crate::Command;

use super::tokens::{Token, TokenType};

pub enum ErrorKind {
    UnexpectedToken,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct Error {
    token: TokenType,
    kind: ErrorKind,
    location: Token,
}

impl Error {
    pub fn new(expected_token: TokenType, kind: ErrorKind, location: Token) -> Self {
        Self {
            token: expected_token,
            kind,
            location,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::UnexpectedToken => f.write_str(
                format!(
                    "Error: {}; expected {:?} after {}",
                    self.kind, self.token, self.location.lexeme
                )
                .as_str(),
            ),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub fn parse(&self) -> Result<Vec<Command>, Error> {
        let i = &self.tokens;
        let (i, parts) = parts(i);

        if parts.is_empty() {
            return Ok(vec![Command::default()]);
        }

        let next = match i.first() {
            Some(token) => token,
            None => return Ok(vec![Command::new(parts[0].clone(), parts[1..].to_vec())]),
        };

        match next.r#type {
            TokenType::AndAnd => todo!(),
            TokenType::And => todo!(),
            TokenType::Part => Err(Error::new(
                TokenType::Part,
                ErrorKind::UnexpectedToken,
                i[0].clone(),
            )),
            TokenType::Eof => Ok(vec![Command::new(parts[0].clone(), parts[1..].to_vec())]),
            TokenType::DollarSign => todo!(),
            TokenType::Pipe => todo!(),
            TokenType::OrOr => todo!(),
            TokenType::Semicolon => todo!(),
            TokenType::LeftBrace => todo!(),
            TokenType::RightBrace => todo!(),
        }
    }
}

#[doc(hidden)]
fn parts(i: &[Token]) -> (Vec<Token>, Vec<String>) {
    let mut result = Vec::new();
    let mut input = i.to_vec();

    for part in input.clone() {
        if let TokenType::Part = part.r#type {
            result.push(part.lexeme);
            input.remove(0);
        }
    }

    (input, result)
}
