use std::{mem::replace, os::macos::raw::stat};

use miette::{SourceOffset, SourceSpan};

use crate::{
    error::ParseError,
    expression::{Expr, Literal},
    statement::{self, Stmt},
    token::{Token, TokenKind},
};

pub struct Parser<'src, I>
where
    I: Iterator<Item = Token>,
{
    source: &'src str,
    tokens: I,
    current: Token,
    previous: Token,
}

impl<'src, I> Parser<'src, I>
where
    I: Iterator<Item = Token>,
{
    #[must_use]
    pub fn new(source: &'src str, mut tokens: I) -> Self {
        let current = tokens
            .next()
            .unwrap_or_else(|| Token::new(TokenKind::EOF, 0, 0));
        Self {
            source,
            tokens,
            previous: current,
            current,
        }
    }

    /// # Errors
    /// Returns a `ParseError` when the parser fails
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.program()
    }

    fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::<Stmt>::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_tokens(&[TokenKind::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        if !self.match_tokens(&[TokenKind::Semicolon]) {
            return Err(ParseError {
                msg: "Expected a semicolon after print statement.".into(),
                span: SourceSpan::new(SourceOffset::from(self.previous.offset), 0),
                help: Some("Add a semicolon after the print statement.".into()),
            });
        }
        Ok(Stmt::PrintStmt(expression))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        if !self.match_tokens(&[TokenKind::Semicolon]) {
            return Err(ParseError {
                msg: "Expected a semicolon after expression statement.".into(),
                span: SourceSpan::new(SourceOffset::from(self.previous.offset), 0),
                help: Some("Add a semicolon after the exression statement.".into()),
            });
        }
        Ok(Stmt::ExprStmt(expression))
    }

    // equality ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous;
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
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut term = self.term()?;

        while self.match_tokens(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous;
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
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut factor = self.factor()?;

        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous;
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
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut unary = self.unary()?;

        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous;
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
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenKind::Minus, TokenKind::Bang]) {
            let operator = self.previous;
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    // NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenKind::False]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }
        if self.match_tokens(&[TokenKind::True]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }
        if self.match_tokens(&[TokenKind::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }

        if self.match_tokens(&[TokenKind::Number(0.0)])
            && let TokenKind::Number(n) = self.previous.kind
        {
            return Ok(Expr::Literal {
                value: Literal::Number(n),
            });
        }

        if self.match_tokens(&[TokenKind::String]) {
            return Ok(Expr::Literal {
                value: Literal::String(self.previous.string_contents(self.source).to_string()),
            });
        }

        if self.match_tokens(&[TokenKind::LeftParen]) {
            let left_paren = self.previous;

            let expr = self.expression()?;
            if self.check(TokenKind::RightParen) {
                let _ = self.advance();
                return Ok(Expr::Grouping {
                    expression: Box::new(expr),
                });
            }
            let _ = self.advance();

            return Err(ParseError {
                msg: "unclosed delimiter".to_string(),
                span: left_paren.as_span(),
                help: Some("Expect ')' after expression to close this group.".to_string()),
            });
        }
        let token = self.peek();
        let lexeme = token.slice(self.source).to_string();
        match token.kind {
            TokenKind::Error(msg) => {
                // Scanner error
                Err(ParseError {
                    msg: format!("{msg}: '{lexeme}'"),
                    span: token.as_span(),
                    help: None,
                })
            }
            TokenKind::EOF => {
                // Scanner error
                Err(ParseError {
                    msg: "Unexpected EOF.".into(),
                    span: SourceSpan::new(SourceOffset::from(self.previous.offset), 0),
                    help: Some("The expression is incomplete.".into()),
                })
            }
            _ => {
                // Scanner error
                Err(ParseError {
                    msg: format!("Unexpected token '{lexeme}'. Expected an expression."),
                    span: self.previous.as_span(),
                    help: Some(
                        "Expressions can be numbers, strings, booleans, or parenthesized groups."
                            .to_string(),
                    ),
                })
            }
        }
    }
}

impl<I> Parser<'_, I>
where
    I: Iterator<Item = Token>,
{
    fn match_tokens(&mut self, types: &[TokenKind]) -> bool {
        for kind in types {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        match (self.peek().kind, kind) {
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (a, b) => a == b,
        }
    }

    fn advance(&mut self) -> Token {
        let next_token = self
            .tokens
            .next()
            .unwrap_or_else(|| Token::new(TokenKind::EOF, self.current.offset, 0));

        self.previous = replace(&mut self.current, next_token);
        self.previous
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    const fn peek(&self) -> Token {
        self.current
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
        let mut parser = Parser::new(expression, scanner);
        let parsed = parser.parse().expect("Failed to parse expression");

        let expected_output = "(== (>= (* (group (+ (- 1) 2)) 3) 4) (! false))";
        assert_eq!(format!("{parsed}"), expected_output);
    }
}
