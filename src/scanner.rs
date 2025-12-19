use crate::token::{Token, TokenKind};

pub struct Scanner<'src> {
    source: &'src str,
    rest: &'src str,
    offset: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            rest: source,
            offset: 0,
        }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.rest.chars().next()?;
        let char_at = self.offset;
        let char_str = &self.rest[..char.len_utf8()];
        self.rest = self.rest.split_at(char.len_utf8()).1;
        self.offset += char.len_utf8();

        let just = move |kind: TokenKind| Some(Ok(Token::new(kind, char_str, char_at)));

        let started = match char {
            '(' => return just(TokenKind::LeftParen),
            ')' => return just(TokenKind::RightParen),
            '{' => return just(TokenKind::LeftBrace),
            '}' => return just(TokenKind::RightBrace),
            ',' => return just(TokenKind::Comma),
            '.' => return just(TokenKind::Dot),
            '-' => return just(TokenKind::Minus),
            '+' => return just(TokenKind::Plus),
            ';' => return just(TokenKind::Semicolon),
            '*' => return just(TokenKind::Star),
            _ => return Some(Err(format!("Unexpected character '{}'", char))),
        };

        None
    }
}
