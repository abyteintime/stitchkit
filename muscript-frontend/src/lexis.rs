mod lexer;
pub mod token;
mod token_stream;

use muscript_foundation::{errors::Diagnostic, source::Span};

pub use lexer::*;
pub use token_stream::*;

pub struct LexError {
    pub diagnostic: Box<Diagnostic>,
    pub span: Span,
}

impl LexError {
    pub fn new(span: Span, diagnostic: Diagnostic) -> Self {
        Self {
            span,
            diagnostic: Box::new(diagnostic),
        }
    }
}

impl From<LexError> for Vec<Diagnostic> {
    fn from(value: LexError) -> Self {
        vec![*value.diagnostic]
    }
}
