use std::mem::replace;

use miette::{SourceOffset, SourceSpan};

use crate::{
    error::ParseError,
    expression::{Expr, Literal},
    statement::{Declaration, Stmt},
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
    pub fn parse(&mut self) -> Result<Vec<Declaration>, ParseError> {
        self.program()
    }

    /// declaration* EOF ;
    fn program(&mut self) -> Result<Vec<Declaration>, ParseError> {
        let mut declarations = Vec::<Declaration>::new();
        while !self.is_at_end() {
            declarations.push(self.declaration()?);
        }
        Ok(declarations)
    }

    /// varDecl | statement
    fn declaration(&mut self) -> Result<Declaration, ParseError> {
        if self.match_tokens(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            Ok(Declaration::Stmt(self.statement()?))
        }
    }

    /// "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Declaration, ParseError> {
        let name = self.consume_identifier("Expected an identifier.")?;
        let initializer = if self.match_tokens(&[TokenKind::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected a semicolon after declaration.",
            Some("Add a semicolon after the declaration."),
        )?;

        Ok(Declaration::VarDecl(name, initializer))
    }

    /// exprStmt | printStmt
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_tokens(&[TokenKind::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    /// "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        self.consume(
            TokenKind::Semicolon,
            "Expected a semicolon after print statement.",
            Some("Add a semicolon after the print statement."),
        )?;
        Ok(Stmt::PrintStmt(expression))
    }

    /// expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        self.consume(
            TokenKind::Semicolon,
            "Expected a semicolon after expression statement.",
            Some("Add a semicolon after the expression statement."),
        )?;
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

    // NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenKind::False]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }
        if self.match_tokens(&[TokenKind::Identifier]) {
            return Ok(Expr::Variable {
                name: self.previous,
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
            self.consume(
                TokenKind::RightParen,
                "unclosed delimiter",
                Some("Expect ')' after expression to close this group."),
            )
            .map_err(|_| ParseError {
                msg: "unclosed delimiter".to_string(),
                span: left_paren.as_span(),
                help: Some("Expect ')' after expression to close this group.".to_string()),
            })?;

            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }
        let token = self.peek();
        let lexeme = token.slice(self.source).to_string();
        match token.kind {
            TokenKind::Error(msg) => {
                // Scanner error
                Err(self.error_at(token, format!("{msg}: '{lexeme}'"), None))
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
                Err(self.error_at(
                    token,
                    format!("Unexpected token '{lexeme}'. Expected an expression."),
                    Some("Expressions can be numbers, strings, booleans, or parenthesized groups."),
                ))
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

    fn consume(
        &mut self,
        kind: TokenKind,
        msg: impl Into<String>,
        help: Option<&str>,
    ) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error_at_current(msg, help))
        }
    }

    fn consume_identifier(&mut self, msg: impl Into<String>) -> Result<String, ParseError> {
        let token = self.peek();
        if token.kind == TokenKind::Identifier {
            self.advance();
            Ok(token.slice(self.source).to_string())
        } else {
            Err(self.error_at_current(msg, None))
        }
    }

    fn error_at_current(&self, msg: impl Into<String>, help: Option<&str>) -> ParseError {
        self.error_at(self.peek(), msg, help)
    }

    fn error_at(&self, token: Token, msg: impl Into<String>, help: Option<&str>) -> ParseError {
        ParseError {
            msg: msg.into(),
            span: token.as_span(),
            help: help.map(str::to_string),
        }
    }

    const fn peek(&self) -> Token {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        scanner::Scanner,
        statement::{Declaration, Stmt},
    };

    use super::*;

    #[test]
    fn test_parser() {
        let source = "(-1 + 2) * 3 >= 4 == !false;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let parsed = parser.parse().expect("Failed to parse program");

        let expected_output = "(== (>= (* (group (+ (- 1) 2)) 3) 4) (! false))";
        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            Declaration::Stmt(Stmt::ExprStmt(expr)) => {
                assert_eq!(format!("{expr}"), expected_output)
            }
            Declaration::Stmt(Stmt::PrintStmt(_)) | Declaration::VarDecl(_, _) => {
                panic!("expected expression statement")
            }
        }
    }

    #[test]
    fn parses_var_declaration_without_initializer() {
        let source = "var breakfast;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let parsed = parser.parse().expect("Failed to parse program");

        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            Declaration::VarDecl(name, None) => assert_eq!(name, "breakfast"),
            _ => panic!("expected variable declaration without initializer"),
        }
    }

    #[test]
    fn parses_var_declaration_with_initializer() {
        let source = "var breakfast = 1 + 2;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let parsed = parser.parse().expect("Failed to parse program");

        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            Declaration::VarDecl(name, Some(expr)) => {
                assert_eq!(name, "breakfast");
                assert_eq!(format!("{expr}"), "(+ 1 2)");
            }
            _ => panic!("expected variable declaration with initializer"),
        }
    }

    #[test]
    fn reports_missing_identifier_in_var_declaration() {
        let source = "var = 1;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let err = parser.parse().expect_err("program should fail to parse");

        assert_eq!(err.msg, "Expected an identifier.");
        assert_eq!(err.span, (4, 1).into());
    }
}
