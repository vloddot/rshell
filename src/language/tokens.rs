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

#[derive(Clone, Debug)]
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
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Eof
    }
}
