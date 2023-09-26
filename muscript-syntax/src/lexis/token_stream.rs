use bitflags::bitflags;
use muscript_foundation::errors::Diagnostic;
use thiserror::Error;

use super::{
    token::{AnyToken, Token},
    LexicalContext,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Channel: u8 {
        /// Main input (everything that is not comments, macros, or errors.)
        const CODE    = 0x1;
        /// Comments only. This is not used by the parser, but it may be used by external tools.
        const COMMENT = 0x2;
        /// Empty macro output. Some rules in the parser recognize this for better error recovery.
        const MACRO   = 0x4;
        /// Lexis errors. Skipped by the parser entirely, though diagnostics from these tokens
        /// are replicated into the output sink.
        const ERROR   = 0x8;
    }
}

pub trait TokenStream {
    fn next_any(&mut self, context: LexicalContext) -> AnyToken;

    fn next_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken {
        loop {
            let token = self.next_any(context);
            if channel.contains(token.kind.channel()) {
                return token;
            }
        }
    }

    fn next(&mut self, context: LexicalContext) -> AnyToken {
        self.next_from(context, Channel::CODE)
    }

    fn peek_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken;

    fn peek_any(&mut self, context: LexicalContext) -> AnyToken {
        self.peek_from(context, Channel::all())
    }

    fn peek(&mut self, context: LexicalContext) -> AnyToken {
        self.peek_from(context, Channel::CODE)
    }

    /// Can be used to add token stream-known context to parser diagnostics.
    fn contextualize_diagnostic(&self, diagnostic: Diagnostic<Token>) -> Diagnostic<Token> {
        diagnostic
    }
}

impl<T> TokenStream for &mut T
where
    T: TokenStream,
{
    fn next_any(&mut self, context: LexicalContext) -> AnyToken {
        <T as TokenStream>::next_any(self, context)
    }

    fn peek_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken {
        <T as TokenStream>::peek_from(self, context, channel)
    }

    fn contextualize_diagnostic(&self, diagnostic: Diagnostic<Token>) -> Diagnostic<Token> {
        <T as TokenStream>::contextualize_diagnostic(self, diagnostic)
    }
}

#[derive(Debug, Error)]
#[error("end of file reached")]
pub struct EofReached;
