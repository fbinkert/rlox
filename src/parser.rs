use std::iter::Peekable;

use crate::{
    expression::{Expr, Literal},
    token::{Token, TokenKind},
};

pub struct Parser<'src, I>
where
    I: Iterator<Item = Result<Token<'src>, String>>,
{
    tokens: Peekable<I>,
    previous: Option<Token<'src>>,
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub token_literal: String,
}

impl<'src, I> Parser<'src, I>
where
    I: Iterator<Item = Result<Token<'src>, String>>,
{
    #[must_use]
    pub fn new(iter: I) -> Self {
        Self {
            tokens: iter.peekable(),
            previous: None,
        }
    }

    /// # Errors
    /// Returns a `ParseError` when the parser fails
    pub fn parse(&mut self) -> Result<Expr<'src>, ParseError> {
        self.expression()
    }

    // equality ;
    fn expression(&mut self) -> Result<Expr<'src>, ParseError> {
        self.equality()
    }

    // comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr<'src>, ParseError> {
        let mut expr = self.comparison()?;

        // Equality expression
        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual])? {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr<'src>, ParseError> {
        let mut term = self.term()?;

        while self.match_tokens(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ])? {
            let operator = self.previous().clone();
            let right = self.term()?;
            term = Expr::Binary {
                left: Box::new(term),
                operator,
                right: Box::new(right),
            };
        }
        Ok(term)
    }

    // factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr<'src>, ParseError> {
        let mut factor = self.factor()?;

        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus])? {
            let operator = self.previous().clone();
            let right = self.factor()?;
            factor = Expr::Binary {
                left: Box::new(factor),
                operator,
                right: Box::new(right),
            };
        }
        Ok(factor)
    }

    // factor -> unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr<'src>, ParseError> {
        let mut unary = self.unary()?;

        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star])? {
            let operator = self.previous().clone();
            let right = self.unary()?;
            unary = Expr::Binary {
                left: Box::new(unary),
                operator,
                right: Box::new(right),
            };
        }

        Ok(unary)
    }

    // unary ( ( "/" | "*" ) unary )* ;
    fn unary(&mut self) -> Result<Expr<'src>, ParseError> {
        if self.match_tokens(&[TokenKind::Minus, TokenKind::Bang])? {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    // NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr<'src>, ParseError> {
        if self.match_tokens(&[TokenKind::False])? {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }
        if self.match_tokens(&[TokenKind::True])? {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }
        if self.match_tokens(&[TokenKind::Nil])? {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }

        if self.match_tokens(&[TokenKind::Number(0.0)])? {
            let prev = self.previous();
            if let TokenKind::Number(n) = prev.kind {
                return Ok(Expr::Literal {
                    value: Literal::Number(n),
                });
            }
        }

        if self.match_tokens(&[TokenKind::String])? {
            let prev = self.previous();
            return Ok(Expr::Literal {
                value: Literal::String(prev.lexeme.to_string()),
            });
        }

        if self.match_tokens(&[TokenKind::LeftParen])? {
            let expr = self.expression()?;
            if self.check(TokenKind::RightParen) {
                let _ = self.advance();
                return Ok(Expr::Grouping {
                    expression: Box::new(expr),
                });
            }
            let token = self.advance()?.clone();
            return Err(Self::error(&token, "Expect ')' after expression."));
        }

        Err(Self::error(self.peek()?, "Expect expression."))
    }
}

// Helper methods

impl<'src, I> Parser<'src, I>
where
    I: Iterator<Item = Result<Token<'src>, String>>,
{
    fn match_tokens(&mut self, types: &[TokenKind]) -> Result<bool, ParseError> {
        for kind in types {
            if self.check(*kind) {
                self.advance()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek()
            .map(|t| match (t.kind, kind) {
                (TokenKind::Number(_), TokenKind::Number(_)) => true,
                (a, b) => a == b,
            })
            .unwrap_or(false)
    }

    fn advance(&mut self) -> Result<&Token<'src>, ParseError> {
        if self.is_at_end() {
            return Ok(self.previous());
        }

        self.peek()?; // Check if next result is an error

        let next_result = self.tokens.next().transpose();
        match next_result {
            Ok(Some(token)) => {
                self.previous = Some(token);
                Ok(self.previous())
            }
            Err(scan_err) => Err(ParseError {
                msg: format!("Scanner error: {scan_err}"),
                token_literal: String::new(),
            }),
            Ok(None) => panic!("Advance called on empty stream"),
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.peek()
            .map(|token| token.kind == TokenKind::EOF)
            .unwrap_or(true)
    }

    fn peek(&mut self) -> Result<&Token<'src>, ParseError> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(token),
            Some(Err(scan_err)) => Err(ParseError {
                msg: format!("Scanner error: {scan_err}"),
                token_literal: "???".to_string(),
            }),
            None => panic!("Parser expected EOF token but got None."),
        }
    }

    const fn previous(&self) -> &Token<'src> {
        self.previous
            .as_ref()
            .expect("Iterator should not be empty")
    }

    fn error(token: &Token<'src>, message: &str) -> ParseError {
        ParseError {
            msg: message.to_string(),
            token_literal: token.lexeme.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn test_parser() {
        let expression = "(-1 + 2) * 3 >= 4 == !false";

        let scanner = Scanner::new(expression);
        let mut parser = Parser::new(scanner);
        let parsed = parser.parse().expect("Failed to parse expression");

        let expected_output = "(== (>= (* (group (+ (- 1) 2)) 3) 4) (! false))";
        assert_eq!(format!("{parsed}"), expected_output);
    }
}
