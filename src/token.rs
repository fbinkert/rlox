use miette::SourceSpan;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number(f64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,

    // Special
    Error(&'static str),
}

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: usize,
    pub length: usize,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind, offset: usize, length: usize) -> Self {
        Self {
            kind,
            offset,
            length,
        }
    }

    #[must_use]
    pub fn as_span(&self) -> SourceSpan {
        (self.offset, self.length).into()
    }

    #[must_use]
    pub fn slice<'src>(&self, source: &'src str) -> &'src str {
        &source[self.offset..self.offset + self.length]
    }

    #[must_use]
    pub fn string_contents<'src>(&self, source: &'src str) -> &'src str {
        debug_assert!(matches!(self.kind, TokenKind::String));
        let lexeme = self.slice(source);
        &lexeme[1..lexeme.len() - 1]
    }
}

impl TokenKind {
    #[must_use]
    pub const fn lexeme(self) -> &'static str {
        match self {
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Minus => "-",
            Self::Plus => "+",
            Self::Semicolon => ";",
            Self::Slash => "/",
            Self::Star => "*",
            Self::Bang => "!",
            Self::BangEqual => "!=",
            Self::Equal => "=",
            Self::EqualEqual => "==",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Identifier => "identifier",
            Self::String => "string",
            Self::Number(_) => "number",
            Self::And => "and",
            Self::Class => "class",
            Self::Else => "else",
            Self::False => "false",
            Self::Fun => "fun",
            Self::For => "for",
            Self::If => "if",
            Self::Nil => "nil",
            Self::Or => "or",
            Self::Print => "print",
            Self::Return => "return",
            Self::Super => "super",
            Self::This => "this",
            Self::True => "true",
            Self::Var => "var",
            Self::While => "while",
            Self::EOF => "",
            Self::Error(_) => "error",
        }
    }
}
