#[derive(Clone, Default, Debug)]
pub struct Token {
    pub location: usize,
    pub r#type: TokenType,
    pub lexeme: String,
}

impl Token {
    #[must_use]
    pub fn new(r#type: TokenType, lexeme: String, location: usize) -> Self {
        Self {
            location,
            r#type,
            lexeme,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    AndAnd,
    And,
    Part,
    DollarSign,
    Pipe,
    OrOr,
    Eof,
    Semicolon,
    LeftBrace,
    RightBrace,
    ColonDash,
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Eof
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AndAnd => "'&&'",
            Self::And => "'&'",
            Self::Part => "identifier",
            Self::DollarSign => "'$'",
            Self::Pipe => "'|'",
            Self::OrOr => "'||'",
            Self::Eof => "<eof>",
            Self::Semicolon => "';'",
            Self::LeftBrace => "'{'",
            Self::RightBrace => "'}'",
            Self::ColonDash => "':-'",
        })
    }
}
