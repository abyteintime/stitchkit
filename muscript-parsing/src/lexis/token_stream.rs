use muscript_foundation::{errors::Diagnostic, source::Span};

use super::{
    token::{Token, TokenKind},
    LexError,
};

pub trait TokenStream {
    fn next_include_comments(&mut self) -> Result<Token, LexError>;

    fn next(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.next_include_comments()?;
            if token.kind != TokenKind::Comment {
                return Ok(token);
            }
        }
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, ()>;

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError>;

    fn peek_include_comments(&mut self) -> Result<Token, LexError>;

    fn peek(&mut self) -> Result<Token, LexError>;

    /// Can be used to add token stream-known context to parser diagnostics.
    fn contextualize_diagnostic(&self, diagnostic: Diagnostic) -> Diagnostic {
        diagnostic
    }
}

impl<T> TokenStream for &mut T
where
    T: TokenStream,
{
    fn next_include_comments(&mut self) -> Result<Token, LexError> {
        <T as TokenStream>::next_include_comments(self)
    }

    /// Parses a "blob", that is any sequence of characters terminated by a character for which
    /// `is_end` returns true. Returns `Err(())` if EOF is reached.
    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, ()> {
        <T as TokenStream>::text_blob(self, is_end)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        <T as TokenStream>::braced_string(self, left_brace_span)
    }

    fn peek_include_comments(&mut self) -> Result<Token, LexError> {
        <T as TokenStream>::peek_include_comments(self)
    }

    fn peek(&mut self) -> Result<Token, LexError> {
        <T as TokenStream>::peek(self)
    }

    fn contextualize_diagnostic(&self, diagnostic: Diagnostic) -> Diagnostic {
        <T as TokenStream>::contextualize_diagnostic(self, diagnostic)
    }
}
