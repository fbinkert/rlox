use miette::NamedSource;

use crate::{error::SpannedError, parser::Parser, scanner::Scanner};

pub mod error;
pub mod expression;
pub mod parser;
pub mod scanner;
pub mod token;

/// # Errors
/// Returns an error if scanning/parsing fails
pub fn run(src: &str) -> miette::Result<()> {
    let scanner = Scanner::new(src);
    let mut parser = Parser::new(scanner);

    match parser.parse() {
        Ok(expr) => {
            println!("{expr}");
        }
        Err(e) => {
            return Err(SpannedError {
                src: NamedSource::new("lox", src.to_string()),
                span: e.span,
                help: e.help.clone(),
                error: e,
            }
            .into());
        }
    }
    Ok(())
}
