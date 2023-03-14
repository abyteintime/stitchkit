#[macro_use]
pub mod token;

mod lexer;
mod peek_cache;
pub mod preprocessor;
mod token_stream;

use muscript_foundation::{errors::Diagnostic, source::Span};

pub use lexer::*;
pub use peek_cache::*;
pub use token_stream::*;

#[derive(Debug, Clone)]
pub struct LexError {
    pub diagnostics: Vec<Diagnostic>,
    pub span: Span,
}

impl LexError {
    pub fn new(span: Span, diagnostic: Diagnostic) -> Self {
        Self {
            span,
            diagnostics: vec![diagnostic],
        }
    }
}

impl From<LexError> for Vec<Diagnostic> {
    fn from(value: LexError) -> Self {
        value.diagnostics
    }
}
