use crate::token::{Token, TokenKind};

pub struct Scanner<'src> {
    source: &'src str,
    rest: &'src str,
    offset: usize,
    reached_eof: bool,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            rest: source,
            offset: 0,
            reached_eof: false,
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

    fn skip_ahead(&mut self, count: usize) {
        self.rest = self.rest.split_at(count).1;
        self.offset += count;
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.rest.chars().next() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.rest.starts_with("//") => {
                    while !self.rest.starts_with('\n') && !self.rest.is_empty() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_comments();

        if self.reached_eof {
            return None;
        }

        let bump = match self.advance() {
            Some(bump) => bump,
            None => {
                self.reached_eof = true;
                return Some(Ok(Token::new(TokenKind::EOF, "", self.offset)));
            }
        };

        enum Started {
            IfEqualThenElse(TokenKind, TokenKind),
            String,
        }

        let token_result =
            move |kind: TokenKind| Some(Ok(Token::new(kind, bump.char_str, bump.char_at)));

        let started = match bump.char {
            '(' => return token_result(TokenKind::LeftParen),
            ')' => return token_result(TokenKind::RightParen),
            '{' => return token_result(TokenKind::LeftBrace),
            '}' => return token_result(TokenKind::RightBrace),
            ',' => return token_result(TokenKind::Comma),
            '.' => return token_result(TokenKind::Dot),
            '-' => return token_result(TokenKind::Minus),
            '+' => return token_result(TokenKind::Plus),
            ';' => return token_result(TokenKind::Semicolon),
            '*' => return token_result(TokenKind::Star),
            '/' => return token_result(TokenKind::Slash),
            '!' => Started::IfEqualThenElse(TokenKind::BangEqual, TokenKind::Bang),
            '=' => Started::IfEqualThenElse(TokenKind::EqualEqual, TokenKind::Equal),
            '<' => Started::IfEqualThenElse(TokenKind::LessEqual, TokenKind::Less),
            '>' => Started::IfEqualThenElse(TokenKind::GreaterEqual, TokenKind::Greater),
            '"' => Started::String,
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
                    token_result(else_)
                }
            }
            Started::String => {
                if let Some(end_offset) = self.rest.find('"') {
                    let start_offset = self.offset;
                    let lexeme = &self.rest[1..end_offset]; // without surrounding quotes

                    self.skip_ahead(end_offset);
                    self.advance(); // consume the closing quote
                    Some(Ok(Token::new(TokenKind::String, lexeme, start_offset)))
                } else {
                    Some(Err("Unterminated string".to_string()))
                }
            }
        }
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
            TokenKind::EOF,
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
            TokenKind::EOF,
        ];

        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_ignore_whitespace() {
        let source = " \t\n\r\n";
        let expected = vec![TokenKind::EOF];

        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_comments() {
        let source = "// this is a comment";
        let expected = vec![TokenKind::EOF];

        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_string_literals() {
        let source = "\"this is a string\" \"this is another string\"";
        let expected = vec![TokenKind::String, TokenKind::String, TokenKind::EOF];

        assert_eq!(scan_to_list(source), expected);
    }
}
