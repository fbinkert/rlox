use crate::scanner::Scanner;

pub mod scanner;
pub mod token;

pub fn run(src: &str) {
    let scanner = Scanner::new(src);
    for token in scanner {
        println!("{:?}", token);
    }
}
