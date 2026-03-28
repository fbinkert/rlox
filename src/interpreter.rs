use std::ops::Deref;

use crate::{
    error::RuntimeError,
    expression::{Expr, Literal},
    token::{Token, TokenKind},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

pub struct Interpreter {}

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
                    TokenKind::Minus => Ok(Value::Number(-(Self::expect_number(&result)?))),
                    TokenKind::Bang => Ok(Value::Bool(!Self::is_truthy(&result))),
                    _ => Err(RuntimeError),
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
                        Self::expect_number(&left_eval)? - Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::Slash => Ok(Value::Number(
                        Self::expect_number(&left_eval)? / Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::Star => Ok(Value::Number(
                        Self::expect_number(&left_eval)? * Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::Plus => match (left_eval, right_eval) {
                        (Value::Number(left_number), Value::Number(right_number)) => {
                            Ok(Value::Number(left_number + right_number))
                        }
                        (Value::String(left_string), Value::String(right_string)) => {
                            Ok(Value::String(left_string + &right_string))
                        }
                        _ => Err(RuntimeError),
                    },
                    TokenKind::Greater => Ok(Value::Bool(
                        Self::expect_number(&left_eval)? > Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::GreaterEqual => Ok(Value::Bool(
                        Self::expect_number(&left_eval)? >= Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::Less => Ok(Value::Bool(
                        Self::expect_number(&left_eval)? < Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::LessEqual => Ok(Value::Bool(
                        Self::expect_number(&left_eval)? <= Self::expect_number(&right_eval)?,
                    )),
                    TokenKind::BangEqual => {
                        Ok(Value::Bool(!Self::is_equal(&left_eval, &right_eval)))
                    }
                    TokenKind::Equal => Ok(Value::Bool(Self::is_equal(&left_eval, &right_eval))),
                    _ => Err(RuntimeError),
                }
            }
        }
    }

    const fn is_truthy(val: &Value) -> bool {
        !matches!(val, Value::Nil | Value::Bool(false))
    }

    const fn expect_number(val: &Value) -> Result<f64, RuntimeError> {
        match val {
            Value::Number(value) => Ok(*value),
            _ => Err(RuntimeError),
        }
    }

    fn is_equal(left: &Value, right: &Value) -> bool {
        left == right
    }
}
