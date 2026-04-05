use miette::NamedSource;

use crate::{
    error::{SpannedError, SpannedRuntimeError},
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
};

pub mod error;
pub mod expression;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod statement;
pub mod token;

/// # Errors
/// Returns an error if scanning, parsing, or evaluation fails
pub fn run(src: &str) -> miette::Result<String> {
    let scanner = Scanner::new(src);
    let mut parser = Parser::new(src, scanner);
    match parser.parse() {
        Ok(expr) => {
            let mut interpreter = Interpreter;
            match interpreter.evaluate(&expr) {
                Ok(value) => Ok(value.to_string()),
                Err(e) => Err(SpannedRuntimeError {
                    src: NamedSource::new("lox", src.to_string()),
                    span: e.span,
                    help: e.help.clone(),
                    error: e,
                }
                .into()),
            }
        }
        Err(e) => Err(SpannedError {
            src: NamedSource::new("lox", src.to_string()),
            span: e.span,
            help: e.help.clone(),
            error: e,
        }
        .into()),
    }
}
