use muscript_foundation::source_arena::SourceArena;
use muscript_lexer::{
    token::{AnyToken, Token, TokenId, TokenKind, TokenSpan},
    token_stream::TokenStream,
};

/// A single element of [`SlicedTokens`].
/// This contains a bit more information than [`TokenSpan`], because the empty token slice contains
/// a source token the slice was constructed from.
#[derive(Debug, Clone, Copy)]
pub enum TokenSlice {
    Empty { source: TokenId },
    Span { start: TokenId, end: TokenId },
}

impl TokenSlice {
    pub fn start(&self) -> TokenId {
        match self {
            TokenSlice::Empty { source } => *source,
            TokenSlice::Span { start, .. } => *start,
        }
    }

    pub fn end(&self) -> TokenId {
        match self {
            TokenSlice::Empty { source } => *source,
            TokenSlice::Span { end, .. } => *end,
        }
    }

    pub fn to_span(&self) -> TokenSpan {
        match *self {
            TokenSlice::Empty { .. } => TokenSpan::Empty,
            TokenSlice::Span { start, end } => TokenSpan::Spanning { start, end },
        }
    }
}

/// Sliced tokens - the data structure and [`TokenStream`] output by the processor.
#[derive(Debug, Clone, Default)]
pub struct SlicedTokens {
    slices: Vec<TokenSlice>,
}

impl SlicedTokens {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, slice: TokenSlice) {
        self.slices.push(slice);
    }

    pub fn stream<'a>(
        &'a self,
        token_arena: &'a SourceArena<Token>,
    ) -> Option<SlicedTokenStream<'a>> {
        Some(SlicedTokenStream {
            token_arena,
            sliced_tokens: self,
            slice_index: 0,
            slice_cursor: self.slices.get(0)?.start(),
        })
    }
}

pub struct SlicedTokenStream<'a> {
    token_arena: &'a SourceArena<Token>,
    sliced_tokens: &'a SlicedTokens,

    slice_index: u32,
    slice_cursor: TokenId,
}

impl<'a> SlicedTokenStream<'a> {
    fn next_slice(&mut self) {
        let previous_slice_index = self.slice_index;
        self.slice_index = (self.slice_index + 1).min(self.sliced_tokens.slices.len() as u32 - 1);
        if self.slice_index != previous_slice_index {
            let current_slice = self.sliced_tokens.slices[self.slice_index as usize];
            match current_slice {
                TokenSlice::Empty { .. } => (),
                TokenSlice::Span { start, .. } => self.slice_cursor = start,
            }
        }
    }
}

impl<'a> TokenStream for SlicedTokenStream<'a> {
    type Position = (u32, TokenId);

    fn next(&mut self) -> AnyToken {
        let result = if let Some(current_slice) =
            self.sliced_tokens.slices.get(self.slice_index as usize)
        {
            match current_slice {
                TokenSlice::Empty { source } => {
                    let token = AnyToken {
                        kind: TokenKind::FailedExp,
                        id: *source,
                    };
                    self.next_slice();
                    token
                }
                TokenSlice::Span { .. } => {
                    let token = self.token_arena.element(self.slice_cursor);
                    let any_token = AnyToken {
                        kind: token.kind,
                        id: self.slice_cursor,
                    };
                    if let Some(next) = self.slice_cursor.successor_in(current_slice.to_span()) {
                        self.slice_cursor = next;
                    } else {
                        self.next_slice();
                    }
                    any_token
                }
            }
        } else {
            AnyToken {
                kind: TokenKind::EndOfFile,
                // Maybe not the best idea to have the last output token pretend to be EOF?
                id: self.slice_cursor,
            }
        };
        result
    }

    fn position(&self) -> Self::Position {
        (self.slice_index, self.slice_cursor)
    }

    fn set_position(&mut self, position: Self::Position) {
        (self.slice_index, self.slice_cursor) = position;
    }
}
