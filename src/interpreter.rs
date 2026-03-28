use crate::{
    error::RuntimeError,
    expression::{Expr, Literal},
    token::{Token, TokenKind},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn evaluate(expr: &Expr<'_>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value } => match value {
                Literal::Number(val) => Ok(Value::Number(*val)),
                Literal::String(val) => Ok(Value::String(val.clone())),
                Literal::Boolean(val) => Ok(Value::Bool(*val)),
                Literal::Nil => Ok(Value::Nil),
            },
            Expr::Grouping { expression } => Self::evaluate(expression), // deref
            // coercion
            Expr::Unary { operator, right } => {
                let result = Self::evaluate(right)?;

                match operator.kind {
                    TokenKind::Minus => Ok(Value::Number(-Self::expect_number(operator, &result)?)),
                    TokenKind::Bang => Ok(Value::Bool(!Self::is_truthy(&result))),
                    _ => Err(Self::error(
                        operator,
                        format!("Unsupported unary operator '{}'.", operator.lexeme),
                        None,
                    )),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_eval = Self::evaluate(left)?;
                let right_eval = Self::evaluate(right)?;

                match operator.kind {
                    TokenKind::Minus => Ok(Value::Number(
                        Self::expect_number(operator, &left_eval)?
                            - Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::Slash => Ok(Value::Number(
                        Self::expect_number(operator, &left_eval)?
                            / Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::Star => Ok(Value::Number(
                        Self::expect_number(operator, &left_eval)?
                            * Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::Plus => match (left_eval, right_eval) {
                        (Value::Number(left_number), Value::Number(right_number)) => {
                            Ok(Value::Number(left_number + right_number))
                        }
                        (Value::String(left_string), Value::String(right_string)) => {
                            Ok(Value::String(left_string + &right_string))
                        }
                        _ => Err(Self::error(
                            operator,
                            "Operands to '+' must be two numbers or two strings.".to_string(),
                            Some(
                                "Try matching the operand types on both sides of '+'.".to_string(),
                            ),
                        )),
                    },
                    TokenKind::Greater => Ok(Value::Bool(
                        Self::expect_number(operator, &left_eval)?
                            > Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::GreaterEqual => Ok(Value::Bool(
                        Self::expect_number(operator, &left_eval)?
                            >= Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::Less => Ok(Value::Bool(
                        Self::expect_number(operator, &left_eval)?
                            < Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::LessEqual => Ok(Value::Bool(
                        Self::expect_number(operator, &left_eval)?
                            <= Self::expect_number(operator, &right_eval)?,
                    )),
                    TokenKind::BangEqual => {
                        Ok(Value::Bool(!Self::is_equal(&left_eval, &right_eval)))
                    }
                    TokenKind::EqualEqual => {
                        Ok(Value::Bool(Self::is_equal(&left_eval, &right_eval)))
                    }
                    _ => Err(Self::error(
                        operator,
                        format!("Unsupported binary operator '{}'.", operator.lexeme),
                        None,
                    )),
                }
            }
        }
    }

    const fn is_truthy(val: &Value) -> bool {
        !matches!(val, Value::Nil | Value::Bool(false))
    }

    fn expect_number(operator: &Token<'_>, val: &Value) -> Result<f64, RuntimeError> {
        match val {
            Value::Number(value) => Ok(*value),
            _ => Err(Self::error(
                operator,
                format!("Operand for '{}' must be a number.", operator.lexeme),
                Some("Use a numeric value here.".to_string()),
            )),
        }
    }

    fn is_equal(left: &Value, right: &Value) -> bool {
        left == right
    }

    fn error(operator: &Token<'_>, msg: String, help: Option<String>) -> RuntimeError {
        RuntimeError {
            msg,
            span: operator.as_span(),
            help,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    use super::Interpreter;

    #[test]
    fn reports_operator_span_for_runtime_errors() {
        let source = "\"a\" - \"b\"";
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let expr = parser.parse().expect("expression should parse");

        let err = Interpreter::evaluate(&expr).expect_err("expression should fail at runtime");

        assert_eq!(err.msg, "Operand for '-' must be a number.");
        assert_eq!(err.help.as_deref(), Some("Use a numeric value here."));
        assert_eq!(err.span, (4, 1).into());
    }
}
