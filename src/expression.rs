use std::fmt;

use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Binary {
        left: Box<Self>,
        operator: Token<'src>,
        right: Box<Self>,
    },
    Grouping {
        expression: Box<Self>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token<'src>,
        right: Box<Self>,
    },
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, operator.lexeme, &[left, right]),
            Expr::Grouping { expression } => parenthesize(f, "group", &[expression]),
            Expr::Literal { value } => write!(f, "{value}"),
            Expr::Unary { operator, right } => parenthesize(f, operator.lexeme, &[right]),
        }
    }
}

fn parenthesize(f: &mut fmt::Formatter<'_>, name: &str, exprs: &[&Expr]) -> fmt::Result {
    write!(f, "({name}")?;
    for expr in exprs {
        write!(f, " {expr}")?;
    }
    write!(f, ")")
}
