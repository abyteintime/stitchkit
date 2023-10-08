use bitflags::bitflags;
use muscript_foundation::{errors::Diagnostic, source_arena::SourceArena, span::Span};
use thiserror::Error;

use crate::token::{AnyToken, TokenId, TokenSpan};

use super::token::Token;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Channel: u8 {
        /// Main input (everything that is not comments, macros, or errors.)
        const CODE    = 0x1;
        /// Comments only. This is not used by the parser, but it may be used by external tools.
        const COMMENT = 0x2;
        /// Whitespace tokens. This is used by the preprocessor to know when linefeeds occur.
        const SPACE   = 0x4;
        /// Empty macro output. Some rules in the parser recognize this for better error recovery.
        const MACRO   = 0x8;
        /// Lexis errors. Skipped by the parser entirely, though diagnostics from these tokens
        /// are replicated into the output sink.
        const ERROR   = 0x16;
    }
}

pub trait TokenStream {
    type Position;

    fn next(&mut self) -> AnyToken;

    fn next_from(&mut self, channel: Channel) -> AnyToken {
        loop {
            let token = self.next();
            if channel.contains(token.kind.channel()) {
                return token;
            }
        }
    }

    fn position(&self) -> Self::Position;

    fn set_position(&mut self, position: Self::Position);

    fn peek(&mut self) -> AnyToken {
        let position = self.position();
        let token = self.next();
        self.set_position(position);
        token
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
    type Position = T::Position;

    fn next(&mut self) -> AnyToken {
        <T as TokenStream>::next(self)
    }

    fn contextualize_diagnostic(&self, diagnostic: Diagnostic<Token>) -> Diagnostic<Token> {
        <T as TokenStream>::contextualize_diagnostic(self, diagnostic)
    }

    fn position(&self) -> Self::Position {
        <T as TokenStream>::position(self)
    }

    fn set_position(&mut self, position: Self::Position) {
        <T as TokenStream>::set_position(self, position)
    }
}

/// [`std::io::Cursor`] but for [`TokenSpan`]s. Turns a [`TokenSpan`] into a [`TokenStream`].
pub struct TokenSpanCursor<'a> {
    token_arena: &'a SourceArena<Token>,
    cursor: TokenId,
    end: TokenId,
}

impl<'a> TokenSpanCursor<'a> {
    /// Returns a cursor for traversing the span, or [`None`] if the span is empty.
    pub fn new(token_arena: &'a SourceArena<Token>, span: TokenSpan) -> Option<Self> {
        match span {
            Span::Empty => None,
            Span::Spanning { start, end } => Some(Self {
                token_arena,
                cursor: start,
                end,
            }),
        }
    }
}

impl<'a> TokenStream for TokenSpanCursor<'a> {
    type Position = TokenId;

    fn next(&mut self) -> AnyToken {
        let id = self.cursor;
        let token = self.token_arena.element(id);
        if let Some(successor) = self
            .cursor
            .successor_in(TokenSpan::spanning(self.cursor, self.end))
        {
            self.cursor = successor;
        }
        AnyToken {
            kind: token.kind,
            id,
        }
    }

    fn position(&self) -> Self::Position {
        self.cursor
    }

    fn set_position(&mut self, position: Self::Position) {
        self.cursor = position;
    }
}

#[derive(Debug, Error)]
#[error("end of file reached")]
pub struct EofReached;
