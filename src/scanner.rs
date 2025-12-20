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

struct Bump<'a> {
    char: char,
    char_at: usize,
    char_str: &'a str,
}

impl<'src> Scanner<'src> {
    fn advance(&mut self) -> Option<Bump<'src>> {
        let char = self.rest.chars().next()?;
        let char_at = self.offset;
        let char_str = &self.rest[..char.len_utf8()];
        self.rest = self.rest.split_at(char.len_utf8()).1;
        self.offset += char.len_utf8();

        Some(Bump {
            char,
            char_at,
            char_str,
        })
    }

    fn next_inner(&mut self) -> Option<Result<Token<'src>, String>> {
        let bump = self.advance()?;

        if bump.char.is_whitespace() {
            return self.next_inner();
        }

        enum Started {
            IfEqualThenElse(TokenKind, TokenKind),
        }

        let just = move |kind: TokenKind| Some(Ok(Token::new(kind, bump.char_str, bump.char_at)));

        let started = match bump.char {
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
            '!' => Started::IfEqualThenElse(TokenKind::BangEqual, TokenKind::Bang),
            '=' => Started::IfEqualThenElse(TokenKind::EqualEqual, TokenKind::Equal),
            '<' => Started::IfEqualThenElse(TokenKind::LessEqual, TokenKind::Less),
            '>' => Started::IfEqualThenElse(TokenKind::GreaterEqual, TokenKind::Greater),
            _ => return Some(Err(format!("Unexpected character '{}'", bump.char))),
        };

        match started {
            Started::IfEqualThenElse(then, else_) => {
                if self.rest.starts_with("=") {
                    let start_offset = bump.char_at;
                    self.advance(); // consume '='
                    let lexeme = &self.source[start_offset..self.offset];
                    Some(Ok(Token::new(then, lexeme, start_offset)))
                } else {
                    just(else_)
                }
            }
            _ => panic!(),
        }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_to_list(source: &str) -> Vec<TokenKind> {
        let scanner = Scanner::new(source);
        scanner
            .map(|res| res.expect("Scanner error"))
            .map(|t| {
                eprintln!("{}", t);
                t.kind
            })
            .collect()
    }

    #[test]
    fn test_single_char_tokens() {
        let source = "( ) { } , . - + ; *";
        let expected = vec![
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Minus,
            TokenKind::Plus,
            TokenKind::Semicolon,
            TokenKind::Star,
        ];
        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_multi_char_operators() {
        let source = "! != = == < <= > >=";
        let expected = vec![
            TokenKind::Bang,
            TokenKind::BangEqual,
            TokenKind::Equal,
            TokenKind::EqualEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ];

        assert_eq!(scan_to_list(source), expected);
    }
}
