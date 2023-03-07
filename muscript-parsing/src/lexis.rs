#[macro_use]
pub mod token;

mod lexer;
pub mod preprocessor;
mod token_stream;

use muscript_foundation::{errors::Diagnostic, source::Span};

pub use lexer::*;
pub use token_stream::*;

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
