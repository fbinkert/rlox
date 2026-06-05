use std::mem::replace;

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

    /// declaration* EOF ;
    fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut declarations = Vec::<Stmt>::new();
        let mut first_error = None;
        while !self.is_at_end() {
            match self.declaration() {
                Ok(declaration) => declarations.push(declaration),
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        first_error.map_or(Ok(declarations), Err)
    }

    /// varDecl | statement
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let result = if self.match_tokens(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronize();
        }

        result
    }

    /// "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected an identifier.")?;
        let initializer = if self.match_tokens(&[TokenKind::Equal]) {
            self.expression()?
        } else {
            // implicit Nil
            Expr::Literal {
                value: Literal::Nil,
            }
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected a semicolon after declaration.",
            Some("Add a semicolon after the declaration."),
        )?;

        Ok(Stmt::VarDecl { name, initializer })
    }

    /// forStmt | ifStmt | printStmt | whileStmt | block | exprStmt
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_tokens(&[TokenKind::For]) {
            self.for_statement()
        } else if self.match_tokens(&[TokenKind::If]) {
            self.if_statement()
        } else if self.match_tokens(&[TokenKind::Print]) {
            self.print_statement()
        } else if self.match_tokens(&[TokenKind::While]) {
            self.while_statement()
        } else if self.match_tokens(&[TokenKind::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    /// "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'for'.", None)?;

        let initializer = if self.match_tokens(&[TokenKind::Semicolon]) {
            None
        } else if self.match_tokens(&[TokenKind::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = (!self.check(TokenKind::Semicolon))
            .then(|| self.expression())
            .transpose()?;

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after loop condition.",
            None,
        )?;

        let increment = (!self.check(TokenKind::RightParen))
            .then(|| self.expression())
            .transpose()?;

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after for clauses.",
            None,
        )?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::ExprStmt(inc)]);
        }

        body = Stmt::WhileStmt {
            condition: condition.unwrap_or(Expr::Literal {
                value: Literal::Boolean(true),
            }),
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    /// "if" "(" expression ")" statement  "else" statement
    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(
            TokenKind::LeftParen,
            "Expected a left parenthesis after if statement.",
            Some("Add parenthesis after if statement."),
        )?;

        let condition = self.expression()?;
        self.consume(
            TokenKind::RightParen,
            "Expected a right parenthesis after if statement.",
            Some("Add closing parenthesis after condtidion."),
        )?;

        let then_branch = Box::new(self.statement()?);

        let else_branch = self
            .match_tokens(&[TokenKind::Else])
            .then(|| self.statement().map(Box::new))
            .transpose()?;

        Ok(Stmt::IfStmt {
            condition,
            then_branch,
            else_branch,
        })
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

    /// "while" "(" expression ")" statement
    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'.", None)?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expected ')' after condition.", None)?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::WhileStmt { condition, body })
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

    /// "{" declaration* "}" ;
    fn block(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace, "Expected '}' after block.", None)?;
        Ok(Stmt::Block(statements))
    }

    // assignment ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // IDENTIFIER "=" assignment | assignment | logic_or;
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expression = self.logic_or()?;

        if self.match_tokens(&[TokenKind::Equal]) {
            let value = self.assignment()?;

            match expression {
                // l-value expressions
                Expr::Variable { name, token } => Ok(Expr::Assign {
                    name,
                    token,
                    value: Box::new(value),
                }),
                _ => Err(ParseError {
                    msg: "invalid assignment target".to_string(),
                    span: self.previous.as_span(),
                    help: None,
                }),
            }
        } else {
            Ok(expression)
        }
    }

    // logic_and ( "or"  logic_and )
    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while self.match_tokens(&[TokenKind::Or]) {
            let operator = self.previous;
            let right = Box::new(self.logic_and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
    }

    // equality ( "and" equality )
    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_tokens(&[TokenKind::And]) {
            let operator = self.previous;
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
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
                token: self.previous,
                name: self.previous.lexeme(self.source).to_string(),
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

    fn synchronize(&mut self) {
        if self.is_at_end() {
            return;
        }

        let _ = self.advance();

        while !self.is_at_end() {
            if self.previous.kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => {
                    let _ = self.advance();
                }
            }
        }
    }

    const fn peek(&self) -> Token {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use crate::{scanner::Scanner, statement::Stmt};

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
            Stmt::ExprStmt(expr) => {
                assert_eq!(format!("{expr}"), expected_output)
            }
            Stmt::PrintStmt(_)
            | Stmt::IfStmt {
                condition: _,
                then_branch: _,
                else_branch: _,
            }
            | Stmt::WhileStmt {
                condition: _,
                body: _,
            }
            | Stmt::Block(_)
            | Stmt::VarDecl {
                name: _,
                initializer: _,
            } => {
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
            Stmt::VarDecl { name, initializer } => {
                assert_eq!(name, "breakfast");
                match initializer {
                    Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Nil)
                    }
                    _ => panic!("expected Nil"),
                }
            }

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
            Stmt::VarDecl { name, initializer } => {
                assert_eq!(name, "breakfast");
                assert_eq!(format!("{initializer}"), "(+ 1 2)");
            }
            _ => panic!("expected variable declaration with initializer"),
        }
    }

    #[test]
    fn desugars_for_loop_into_initializer_and_while() {
        let source = "for (var i = 0; i < 3; i = i + 1) print i;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let parsed = parser.parse().expect("Failed to parse program");

        assert_eq!(parsed.len(), 1);
        match &parsed[0] {
            Stmt::Block(statements) => {
                assert_eq!(statements.len(), 2);

                match &statements[0] {
                    Stmt::VarDecl { name, initializer } => {
                        assert_eq!(name, "i");
                        assert_eq!(format!("{initializer}"), "0");
                    }
                    _ => panic!("expected loop initializer"),
                }

                match &statements[1] {
                    Stmt::WhileStmt { condition, body } => {
                        assert_eq!(format!("{condition}"), "(< i 3)");

                        match body.as_ref() {
                            Stmt::Block(loop_body) => {
                                assert_eq!(loop_body.len(), 2);
                                match &loop_body[0] {
                                    Stmt::PrintStmt(expr) => {
                                        assert_eq!(format!("{expr}"), "i");
                                    }
                                    _ => panic!("expected original loop body"),
                                }
                                match &loop_body[1] {
                                    Stmt::ExprStmt(expr) => {
                                        assert_eq!(format!("{expr}"), "(i (+ i 1))");
                                    }
                                    _ => panic!("expected increment expression"),
                                }
                            }
                            _ => panic!("expected increment to be appended to loop body"),
                        }
                    }
                    _ => panic!("expected desugared while loop"),
                }
            }
            _ => panic!("expected desugared for loop block"),
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

    #[test]
    fn synchronizes_after_failed_declaration() {
        let source = "var = 1; print 2;";

        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);

        let err = parser
            .declaration()
            .expect_err("first declaration should fail");
        assert_eq!(err.msg, "Expected an identifier.");

        let declaration = parser
            .declaration()
            .expect("parser should recover to the next declaration");
        match declaration {
            Stmt::PrintStmt(expr) => assert_eq!(format!("{expr}"), "2"),
            _ => panic!("expected recovered print statement"),
        }
    }
}
