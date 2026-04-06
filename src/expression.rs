use std::fmt;

use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", format_number(*n)),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: String,
        token: Token,
        value: Box<Self>,
    },
    Binary {
        left: Box<Self>,
        operator: Token,
        right: Box<Self>,
    },
    Grouping {
        expression: Box<Self>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Self>,
    },
    Variable {
        name: String,
        token: Token,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, operator.kind.lexeme(), &[left, right]),
            Self::Grouping { expression } => parenthesize(f, "group", &[expression]),
            Self::Literal { value } => write!(f, "{value}"),
            Self::Unary { operator, right } => parenthesize(f, operator.kind.lexeme(), &[right]),
            Self::Assign { name, token, value } => parenthesize(f, name, &[value]),
            Self::Variable { name, token } => write!(f, "{}", name),
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

fn format_number(n: f64) -> String {
    let mut text = n.to_string();
    if text.ends_with(".0") {
        text.truncate(text.len() - 2);
    }
    text
}
