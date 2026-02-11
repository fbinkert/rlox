use crate::{parser::Parser, scanner::Scanner};

pub mod expression;
pub mod parser;
pub mod scanner;
pub mod token;

pub fn run(src: &str) {
    let scanner = Scanner::new(src);
    let mut parser = Parser::new(scanner);

    match parser.parse() {
        Ok(expr) => {
            println!("{expr}");
        }
        Err(e) => eprintln!("Error: {}", e.msg),
    }
}
