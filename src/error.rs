#![allow(unused_assignments)]

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{msg}")]
pub struct ParseError {
    pub msg: String,
    pub span: SourceSpan,
    pub help: Option<String>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Syntax Error")]
#[diagnostic(code(lox::parser))]
pub struct SpannedError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("{error}")]
    pub span: SourceSpan,

    #[help]
    pub help: Option<String>,

    pub error: ParseError,
}
