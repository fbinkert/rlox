use crate::token::{Token, TokenKind};

pub struct Scanner<'src> {
    source: &'src str,
    rest: &'src str,
    // byte offset of 'rest' relative to 'source'
    byte_offset: usize,
    eof_emitted: bool,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            rest: source,
            byte_offset: 0,
            eof_emitted: false,
        }
    }
}

impl<'src> Scanner<'src> {
    /// Returns the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.rest.chars().next()
    }

    /// Returns character after the next one  without consuming it (lookahead 1)
    fn peek_next(&mut self) -> Option<char> {
        let mut chars = self.rest.chars();
        chars.next();
        chars.next()
    }

    /// Consumes the next character and returns it
    fn advance(&mut self) -> Option<char> {
        let c = self.rest.chars().next()?;
        self.rest = self.rest.split_at(c.len_utf8()).1;
        self.byte_offset += c.len_utf8();
        Some(c)
    }

    /// Conditional advance
    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            return true;
        }
        false
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    if let Some(pos) = self.rest.find('\n') {
                        let skip = pos + 1;
                        self.byte_offset += skip;
                        self.rest = &self.rest[skip..];
                    } else {
                        self.byte_offset += self.rest.len();
                        self.rest = ""
                    };

                    while !self.rest.starts_with('\n') && !self.rest.is_empty() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn make_token(
        &mut self,
        kind: TokenKind,
        start_offset: usize,
    ) -> Option<Result<Token<'src>, String>> {
        Some(Ok(Token::new(
            kind,
            &self.source[start_offset..self.byte_offset],
            start_offset,
        )))
    }

    fn consume_digits(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn consume_ident(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
    }
}

impl<'src> Scanner<'src> {
    fn scan_string_literal(&mut self, start_offset: usize) -> Option<Result<Token<'src>, String>> {
        if let Some(byte_length) = self.rest.find('"') {
            let start_content = self.byte_offset + 1; // without starting quote
            let end_content = self.byte_offset + byte_length; // without ending quote
            let lexeme = &self.source[start_content..end_content];

            let advance_by = byte_length + 1;
            self.byte_offset += advance_by;
            self.rest = &self.rest[advance_by..];
            Some(Ok(Token::new(TokenKind::String, lexeme, start_offset)))
        } else {
            Some(Err("Unterminated string".to_string()))
        }
    }

    fn scan_number(&mut self, start_offset: usize) -> Option<Result<Token<'src>, String>> {
        self.consume_digits();

        if let Some(c) = self.peek_next()
            && self.peek() == Some('.')
            && c.is_ascii_digit()
        {
            self.advance(); // consume '.'
            self.consume_digits();
        }

        let lexeme = &self.source[start_offset..self.byte_offset];

        let number = match lexeme.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return Some(Err(format!("Invalid number literal: {}", lexeme))),
        };

        Some(Ok(Token::new(
            TokenKind::Number(number),
            lexeme,
            start_offset,
        )))
    }

    fn scan_ident(&mut self, start_offset: usize) -> Option<Result<Token<'src>, String>> {
        self.consume_ident();
        let lexeme = &self.source[start_offset..self.byte_offset];
        let kind = Self::get_keyword_kind(lexeme).unwrap_or(TokenKind::Identifier);
        Some(Ok(Token::new(kind, lexeme, start_offset)))
    }

    fn get_keyword_kind(lexeme: &str) -> Option<TokenKind> {
        match lexeme {
            "and" => Some(TokenKind::And),
            "class" => Some(TokenKind::Class),
            "else" => Some(TokenKind::Else),
            "false" => Some(TokenKind::False),
            "for" => Some(TokenKind::For),
            "fun" => Some(TokenKind::Fun),
            "if" => Some(TokenKind::If),
            "nil" => Some(TokenKind::Nil),
            "or" => Some(TokenKind::Or),
            "print" => Some(TokenKind::Print),
            "return" => Some(TokenKind::Return),
            "super" => Some(TokenKind::Super),
            "this" => Some(TokenKind::This),
            "true" => Some(TokenKind::True),
            "var" => Some(TokenKind::Var),
            "while" => Some(TokenKind::While),
            _ => None,
        }
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if self.eof_emitted {
            return None;
        }

        if self.rest.is_empty() {
            if self.eof_emitted {
                return None;
            };
            self.eof_emitted = true;
            return Some(Ok(Token::new(TokenKind::EOF, "", self.byte_offset)));
        }

        let start_offset = self.byte_offset;

        match self.advance()? {
            '(' => self.make_token(TokenKind::LeftParen, start_offset),
            ')' => self.make_token(TokenKind::RightParen, start_offset),
            '{' => self.make_token(TokenKind::LeftBrace, start_offset),
            '}' => self.make_token(TokenKind::RightBrace, start_offset),
            ',' => self.make_token(TokenKind::Comma, start_offset),
            '.' => self.make_token(TokenKind::Dot, start_offset),
            '-' => self.make_token(TokenKind::Minus, start_offset),
            '+' => self.make_token(TokenKind::Plus, start_offset),
            ';' => self.make_token(TokenKind::Semicolon, start_offset),
            '*' => self.make_token(TokenKind::Star, start_offset),
            '/' => self.make_token(TokenKind::Slash, start_offset),
            '!' => {
                let kind = if self.matches('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.make_token(kind, start_offset)
            }

            '=' => {
                let kind = if self.matches('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.make_token(kind, start_offset)
            }

            '>' => {
                let kind = if self.matches('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.make_token(kind, start_offset)
            }

            '<' => {
                let kind = if self.matches('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.make_token(kind, start_offset)
            }

            '"' => self.scan_string_literal(start_offset),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_ident(start_offset),
            '0'..='9' => self.scan_number(start_offset),
            c => Some(Err(format!("Unexpected character '{}'", c))),
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

    #[test]
    fn test_number() {
        let source = "123 123.456";
        let expected = vec![
            TokenKind::Number(123.0),
            TokenKind::Number(123.456),
            TokenKind::EOF,
        ];

        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_ident() {
        let source = "and if else";
        let expected = vec![
            TokenKind::And,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::EOF,
        ];

        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_number_with_trailing_dot() {
        let source = "123.";
        let expected = vec![TokenKind::Number(123.0), TokenKind::Dot, TokenKind::EOF];
        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_number_method_call() {
        let source = "123.toString";
        let expected = vec![
            TokenKind::Number(123.0),
            TokenKind::Dot,
            TokenKind::Identifier,
            TokenKind::EOF,
        ];
        assert_eq!(scan_to_list(source), expected);
    }

    #[test]
    fn test_standard_float() {
        let source = "123.456";
        let expected = vec![TokenKind::Number(123.456), TokenKind::EOF];
        assert_eq!(scan_to_list(source), expected);
    }
}
