use muscript_foundation::source_arena::SourceArena;

use crate::{
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

    pub fn push_slice(&mut self, slice: TokenSlice) {
        self.slices.push(slice);
    }

    pub fn push_token(&mut self, token: TokenId) {
        match &mut self.slices[..] {
            [.., TokenSlice::Span { start: _, end }] if end.successor() == token => {
                *end = token;
            }
            _ => {
                self.slices.push(TokenSlice::Span {
                    start: token,
                    end: token,
                });
            }
        }
    }

    pub fn stream<'a>(
        &'a self,
        token_arena: &'a SourceArena<Token>,
    ) -> Option<SlicedTokenStream<'a>> {
        Some(SlicedTokenStream {
            token_arena,
            sliced_tokens: self,
            slice_index: 0,
            cursor: self.slices.get(0)?.start(),
        })
    }
}

pub struct SlicedTokenStream<'a> {
    token_arena: &'a SourceArena<Token>,
    sliced_tokens: &'a SlicedTokens,

    slice_index: u32,
    cursor: TokenId,
}

impl<'a> SlicedTokenStream<'a> {
    fn next_slice(&mut self) {
        self.slice_index += 1;
        if let Some(&TokenSlice::Span { start, .. }) =
            self.sliced_tokens.slices.get(self.slice_index as usize)
        {
            self.cursor = start;
        }
    }
}

impl<'a> TokenStream for SlicedTokenStream<'a> {
    type Position = (u32, TokenId);

    fn next(&mut self) -> AnyToken {
        let result =
            if let Some(current_slice) = self.sliced_tokens.slices.get(self.slice_index as usize) {
                match current_slice {
                    TokenSlice::Empty { source } => {
                        let token = AnyToken {
                            kind: TokenKind::FailedExp,
                            id: *source,
                        };
                        self.next_slice();
                        token
                    }
                    TokenSlice::Span { start: _, end } => {
                        let kind = self.token_arena.element(self.cursor).kind;
                        let id = self.cursor;
                        self.cursor = self.cursor.successor();
                        if self.cursor > *end {
                            self.next_slice();
                        }
                        AnyToken { kind, id }
                    }
                }
            } else {
                AnyToken {
                    kind: TokenKind::EndOfFile,
                    // self.cursor is always going to be one past the last token in this case
                    id: self.cursor.predecessor().unwrap(),
                }
            };
        result
    }

    fn position(&self) -> Self::Position {
        (self.slice_index, self.cursor)
    }

    fn set_position(&mut self, position: Self::Position) {
        (self.slice_index, self.cursor) = position;
    }
}

#[cfg(test)]
mod tests {
    use muscript_foundation::{
        source::{SourceFile, SourceFileSet},
        source_arena::SourceArena,
    };

    use crate::{
        token::{Token, TokenKind, TokenSpan},
        token_stream::TokenStream,
    };

    use super::{SlicedTokens, TokenSlice};

    #[test]
    fn iteration_over_many_slices() {
        let mut source_file_set = SourceFileSet::new();
        let mut token_arena = SourceArena::<Token>::new();

        let mut builder = token_arena.build_source_file(source_file_set.add(SourceFile::new(
            "TestPackage".into(),
            "Test.uc".into(),
            "Test.uc".into(),
            "".into(),
        )));
        builder.push(Token {
            kind: TokenKind::Ident,
            source_range: 0..1,
        });
        builder.push(Token {
            kind: TokenKind::IntLit,
            source_range: 1..2,
        });
        let TokenSpan::Spanning {
            start: start_1,
            end: end_1,
        } = builder.finish()
        else {
            unreachable!()
        };

        let mut builder = token_arena.build_source_file(source_file_set.add(SourceFile::new(
            "TestPackage".into(),
            "Test.uc".into(),
            "Test.uc".into(),
            "".into(),
        )));
        builder.push(Token {
            kind: TokenKind::Equal,
            source_range: 0..1,
        });
        builder.push(Token {
            kind: TokenKind::NotEqual,
            source_range: 1..2,
        });
        let TokenSpan::Spanning {
            start: start_2,
            end: end_2,
        } = builder.finish()
        else {
            unreachable!()
        };

        let mut sliced_tokens = SlicedTokens::new();
        sliced_tokens.push_slice(TokenSlice::Span {
            start: start_1,
            end: end_1,
        });
        sliced_tokens.push_slice(TokenSlice::Span {
            start: start_2,
            end: end_2,
        });
        sliced_tokens.push_slice(TokenSlice::Span {
            start: start_1,
            end: end_1,
        });
        sliced_tokens.push_slice(TokenSlice::Empty { source: end_2 });
        sliced_tokens.push_slice(TokenSlice::Span {
            start: start_1,
            end: end_1,
        });

        let mut stream = sliced_tokens
            .stream(&token_arena)
            .expect("SlicedTokens are not empty");

        assert_eq!(stream.next().kind, TokenKind::Ident);
        assert_eq!(stream.next().kind, TokenKind::IntLit);

        assert_eq!(stream.next().kind, TokenKind::Equal);
        assert_eq!(stream.next().kind, TokenKind::NotEqual);

        assert_eq!(stream.next().kind, TokenKind::Ident);
        assert_eq!(stream.next().kind, TokenKind::IntLit);

        assert_eq!(stream.next().kind, TokenKind::FailedExp);

        assert_eq!(stream.next().kind, TokenKind::Ident);
        assert_eq!(stream.next().kind, TokenKind::IntLit);

        assert_eq!(stream.next().kind, TokenKind::EndOfFile);
    }
}
