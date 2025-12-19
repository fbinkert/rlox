use crate::token::Token;

pub struct Scanner {}

impl Scanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn scan_tokens(&self, source: &str) -> Vec<Token> {
        Vec::new()
    }
}
