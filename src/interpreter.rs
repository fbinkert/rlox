use crate::{
    environment::Environment,
    error::RuntimeError,
    expression::{Expr, Literal},
    statement::Stmt,
    token::{Token, TokenKind},
};
use std::{cell::RefCell, fmt, rc::Rc};

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
            Self::Number(n) => write!(f, "{}", format_number(*n)),
            Self::String(s) => write!(f, "{s}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl From<&Literal> for Value {
    fn from(value: &Literal) -> Self {
        match value {
            Literal::Number(val) => Self::Number(*val),
            Literal::String(val) => Self::String(val.clone()),
            Literal::Boolean(val) => Self::Bool(*val),
            Literal::Nil => Self::Nil,
        }
    }
}

fn format_number(n: f64) -> String {
    let mut text = n.to_string();
    if text.ends_with(".0") {
        text.truncate(text.len() - 2);
    }
    text
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    /// # Errors
    /// Raises a runtime error on error
    pub fn interpret(&mut self, program: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in program {
            match statement {
                Stmt::PrintStmt(expression) => {
                    let value = self.evaluate(expression)?;
                    println!("{value}");
                }
                Stmt::ExprStmt(expression) => {
                    self.evaluate(expression)?;
                }
                Stmt::VarDecl { name, initializer } => {
                    let value = self.evaluate(initializer)?;
                    self.environment.borrow_mut().define(name.clone(), value);
                }
                Stmt::Block(stmts) => {
                    self.execute_block(stmts, Environment::new_enclosed(self.environment.clone()))?;
                }
            }
        }

        Ok(())
    }

    fn execute_block(&mut self, stmts: &[Stmt], env: Environment) -> Result<(), RuntimeError> {
        let parent_env = self.environment.clone();
        self.environment = Rc::new(RefCell::new(env));
        let result = self.interpret(stmts);
        self.environment = parent_env;
        result
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Assign { name, token, value } => {
                if self.environment.borrow_mut().contains(name) {
                    let value = self.evaluate(value)?;
                    self.environment
                        .borrow_mut()
                        .assign(name.clone(), value.clone());
                    Ok(value)
                } else {
                    Err(RuntimeError {
                        msg: "Undefined variable".to_string(),
                        span: token.as_span(),
                        help: None,
                    })
                }
            }
            Expr::Variable { name, token } => {
                let maybe_value = self.environment.borrow_mut().get(name);
                maybe_value.map_or_else(
                    || {
                        Err(RuntimeError {
                            msg: "Undefined variable".to_string(),
                            span: token.as_span(),
                            help: None,
                        })
                    },
                    Ok,
                )
            }
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let result = self.evaluate(right)?;

                match operator.kind {
                    TokenKind::Minus => Ok(Value::Number(-self.expect_number(operator, &result)?)),
                    TokenKind::Bang => Ok(Value::Bool(!Self::is_truthy(&result))),
                    _ => Err(Self::error(
                        operator,
                        format!("Unsupported unary operator '{}'.", operator.kind.lexeme()),
                        None,
                    )),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_eval = self.evaluate(left)?;
                let right_eval = self.evaluate(right)?;

                match operator.kind {
                    TokenKind::Minus => self.eval_numeric_binary(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| Value::Number(left - right),
                    ),
                    TokenKind::Slash => self.eval_numeric_binary(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| Value::Number(left / right),
                    ),
                    TokenKind::Star => self.eval_numeric_binary(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| Value::Number(left * right),
                    ),
                    TokenKind::Plus => self.eval_plus(operator, left_eval, right_eval),
                    TokenKind::Greater => self.eval_numeric_comparison(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| left > right,
                    ),
                    TokenKind::GreaterEqual => self.eval_numeric_comparison(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| left >= right,
                    ),
                    TokenKind::Less => self.eval_numeric_comparison(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| left < right,
                    ),
                    TokenKind::LessEqual => self.eval_numeric_comparison(
                        operator,
                        &left_eval,
                        &right_eval,
                        |left, right| left <= right,
                    ),
                    TokenKind::BangEqual => {
                        Ok(Value::Bool(!Self::is_equal(&left_eval, &right_eval)))
                    }
                    TokenKind::EqualEqual => {
                        Ok(Value::Bool(Self::is_equal(&left_eval, &right_eval)))
                    }
                    _ => Err(Self::error(
                        operator,
                        format!("Unsupported binary operator '{}'.", operator.kind.lexeme()),
                        None,
                    )),
                }
            }
        }
    }

    const fn is_truthy(val: &Value) -> bool {
        !matches!(val, Value::Nil | Value::Bool(false))
    }

    fn expect_number(&self, operator: &Token, val: &Value) -> Result<f64, RuntimeError> {
        match val {
            Value::Number(value) => Ok(*value),
            _ => Err(Self::error(
                operator,
                format!("Operand to '{}' must be a number.", operator.kind.lexeme()),
                Some("Use a numeric value with this unary operator.".to_string()),
            )),
        }
    }

    fn expect_numbers(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
    ) -> Result<(f64, f64), RuntimeError> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok((*left, *right)),
            _ => Err(Self::error(
                operator,
                format!("Operands to '{}' must be numbers.", operator.kind.lexeme()),
                Some("Use numeric values on both sides of the operator.".to_string()),
            )),
        }
    }

    fn is_equal(left: &Value, right: &Value) -> bool {
        left == right
    }

    fn eval_numeric_binary<F>(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
        f: F,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> Value,
    {
        let (left, right) = self.expect_numbers(operator, left, right)?;
        Ok(f(left, right))
    }

    fn eval_numeric_comparison<F>(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
        f: F,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let (left, right) = self.expect_numbers(operator, left, right)?;
        Ok(Value::Bool(f(left, right)))
    }

    fn eval_plus(
        &self,
        operator: &Token,
        left: Value,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
            (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
            _ => Err(Self::error(
                operator,
                "Operands to '+' must be two numbers or two strings.".to_string(),
                Some("Try matching the operand types on both sides of '+'.".to_string()),
            )),
        }
    }

    fn error(operator: &Token, msg: String, help: Option<String>) -> RuntimeError {
        RuntimeError {
            msg,
            span: operator.as_span(),
            help,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner, statement::Stmt};

    use super::{Interpreter, Value};

    fn parse_expression(source: &str) -> crate::expression::Expr {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let program = parser.parse().expect("program should parse");

        assert_eq!(program.len(), 1, "expected a single statement");
        match program.into_iter().next().expect("statement should exist") {
            Stmt::ExprStmt(expr) => expr,
            Stmt::PrintStmt(_)
            | Stmt::Block(_)
            | Stmt::VarDecl {
                name: _,
                initializer: _,
            } => {
                panic!("expected expression statement")
            }
        }
    }

    fn evaluate(source: &str) -> Value {
        let expr = parse_expression(source);
        let mut interpreter = Interpreter::new();

        interpreter
            .evaluate(&expr)
            .expect("expression should evaluate")
    }

    fn evaluate_error(source: &str) -> crate::error::RuntimeError {
        let expr = parse_expression(source);
        let mut interpreter = Interpreter::new();

        interpreter
            .evaluate(&expr)
            .expect_err("expression should fail at runtime")
    }

    fn interpret_program(source: &str) -> Interpreter {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(source, scanner);
        let program = parser.parse().expect("program should parse");
        let mut interpreter = Interpreter::new();

        interpreter
            .interpret(&program)
            .expect("program should interpret");

        interpreter
    }

    #[test]
    fn evaluates_truthiness_for_bang() {
        assert_eq!(evaluate("!nil;"), Value::Bool(true));
    }

    #[test]
    fn evaluates_nested_equality() {
        assert_eq!(evaluate("!(false == true);"), Value::Bool(true));
    }

    #[test]
    fn concatenates_strings() {
        assert_eq!(evaluate("\"a\" + \"b\";"), Value::String("ab".to_string()));
    }

    #[test]
    fn evaluates_comparisons() {
        assert_eq!(evaluate("1 < 2 == true;"), Value::Bool(true));
    }

    #[test]
    fn reports_operator_span_for_runtime_errors() {
        let err = evaluate_error("\"a\" - \"b\";");

        assert_eq!(err.msg, "Operands to '-' must be numbers.");
        assert_eq!(
            err.help.as_deref(),
            Some("Use numeric values on both sides of the operator.")
        );
        assert_eq!(err.span, (4, 1).into());
    }

    #[test]
    fn reports_plus_type_mismatch() {
        let err = evaluate_error("\"a\" + 1;");

        assert_eq!(
            err.msg,
            "Operands to '+' must be two numbers or two strings."
        );
    }

    #[test]
    fn block_can_read_outer_variable() {
        let scanner = Scanner::new("var a = 1; { a; }");
        let mut parser = Parser::new("var a = 1; { a; }", scanner);
        let program = parser.parse().expect("program should parse");
        let mut interpreter = Interpreter::new();

        interpreter
            .interpret(&program)
            .expect("block should read outer variable");
    }

    #[test]
    fn assignment_in_block_updates_outer_variable() {
        let interpreter = interpret_program("var a = 1; { a = 2; }");

        assert_eq!(
            interpreter.environment.borrow().get("a"),
            Some(Value::Number(2.0))
        );
    }

    #[test]
    fn block_variable_shadowing_does_not_modify_outer_variable() {
        let interpreter = interpret_program("var a = 1; { var a = 2; }");

        assert_eq!(
            interpreter.environment.borrow().get("a"),
            Some(Value::Number(1.0))
        );
    }
}
