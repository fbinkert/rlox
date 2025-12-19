use std::fmt::{Display, Formatter};

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
}

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub lexeme: &'src str,
    pub offset: usize,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind, lexeme: &'src str, offset: usize) -> Token<'src> {
        Token {
            kind,
            lexeme,
            offset,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;
        match self.kind {
            LeftParen => write!(f, "LEFT_PAREN {}", self.lexeme),
            RightParen => write!(f, "RIGHT_PAREN {}", self.lexeme),
            LeftBrace => write!(f, "LEFT_BRACE {}", self.lexeme),
            RightBrace => write!(f, "RIGHT_BRACE {}", self.lexeme),
            Comma => write!(f, "COMMA {}", self.lexeme),
            Dot => write!(f, "DOT {}", self.lexeme),
            Minus => write!(f, "MINUS {}", self.lexeme),
            Plus => write!(f, "PLUS {}", self.lexeme),
            Semicolon => write!(f, "SEMICOLON {}", self.lexeme),
            Slash => write!(f, "SLASH {}", self.lexeme),
            Star => write!(f, "STAR {}", self.lexeme),

            // One or two character tokens
            Bang => write!(f, "BANG {}", self.lexeme),
            BangEqual => write!(f, "BANG_EQUAL {}", self.lexeme),
            Equal => write!(f, "EQUAL {}", self.lexeme),
            EqualEqual => write!(f, "EQUAL_EQUAL {}", self.lexeme),
            Greater => write!(f, "GREATER {}", self.lexeme),
            GreaterEqual => write!(f, "GREATER_EQUAL {}", self.lexeme),
            Less => write!(f, "LESS {}", self.lexeme),
            LessEqual => write!(f, "LESS_EQUAL {}", self.lexeme),

            // Literals
            Identifier => write!(f, "IDENTIFIER {}", self.lexeme),
            String => write!(f, "STRING {}", self.lexeme),
            Number(n) => write!(f, "NUMBER {} {}", self.lexeme, n),

            // Keywords
            And => write!(f, "AND {}", self.lexeme),
            Class => write!(f, "CLASS {}", self.lexeme),
            Else => write!(f, "ELSE {}", self.lexeme),
            False => write!(f, "FALSE {}", self.lexeme),
            Fun => write!(f, "FUN {}", self.lexeme),
            For => write!(f, "FOR {}", self.lexeme),
            If => write!(f, "IF {}", self.lexeme),
            Nil => write!(f, "NIL {}", self.lexeme),
            Or => write!(f, "OR {}", self.lexeme),
            Print => write!(f, "PRINT {}", self.lexeme),
            Return => write!(f, "RETURN {}", self.lexeme),
            Super => write!(f, "SUPER {}", self.lexeme),
            This => write!(f, "THIS {}", self.lexeme),
            True => write!(f, "TRUE {}", self.lexeme),
            Var => write!(f, "VAR {}", self.lexeme),
            While => write!(f, "WHILE {}", self.lexeme),

            EOF => write!(f, "EOF {}", self.lexeme),
        }
    }
}
