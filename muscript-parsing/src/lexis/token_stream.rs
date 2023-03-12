use bitflags::bitflags;
use muscript_foundation::{errors::Diagnostic, source::Span};
use thiserror::Error;

use super::{token::Token, LexError};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Channel: u8 {
        /// Main input (everything that is not comments or macros.)
        const MAIN     = 0x1;
        /// Comments only. This is not used by the parser, but it may be used by external tools.
        const COMMENTS = 0x2;
        /// Empty macro output. Some rules in the parser recognize this for better error recovery.
        const MACROS   = 0x4;
    }
}

pub trait TokenStream {
    fn next_any(&mut self) -> Result<Token, LexError>;

    fn next_from(&mut self, channel: Channel) -> Result<Token, LexError> {
        loop {
            let token = self.next_any()?;
            if channel.contains(token.kind.channel()) {
                return Ok(token);
            }
        }
    }

    fn next(&mut self) -> Result<Token, LexError> {
        self.next_from(Channel::MAIN)
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached>;

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError>;

    fn peek_from(&mut self, channel: Channel) -> Result<Token, LexError>;

    fn peek_any(&mut self) -> Result<Token, LexError> {
        self.peek_from(Channel::all())
    }

    fn peek(&mut self) -> Result<Token, LexError> {
        self.peek_from(Channel::MAIN)
    }

    /// Can be used to add token stream-known context to parser diagnostics.
    fn contextualize_diagnostic(&self, diagnostic: Diagnostic) -> Diagnostic {
        diagnostic
    }
}

impl<T> TokenStream for &mut T
where
    T: TokenStream,
{
    fn next_any(&mut self) -> Result<Token, LexError> {
        <T as TokenStream>::next_any(self)
    }

    /// Parses a "blob", that is any sequence of characters terminated by a character for which
    /// `is_end` returns true. Returns `Err(())` if EOF is reached.
    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached> {
        <T as TokenStream>::text_blob(self, is_end)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        <T as TokenStream>::braced_string(self, left_brace_span)
    }

    fn peek_from(&mut self, channel: Channel) -> Result<Token, LexError> {
        <T as TokenStream>::peek_from(self, channel)
    }

    fn contextualize_diagnostic(&self, diagnostic: Diagnostic) -> Diagnostic {
        <T as TokenStream>::contextualize_diagnostic(self, diagnostic)
    }
}

#[derive(Debug, Error)]
#[error("end of file reached")]
pub struct EofReached;
