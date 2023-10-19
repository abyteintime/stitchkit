use std::ops::Range;

use muscript_foundation::{
    errors::ReplacementSuggestion,
    source::SourceFileSet,
    source_arena::SourceArena,
    span::{Span, Spanned},
};

use crate::token::{Token, TokenId};

#[derive(Clone, Copy)]
pub struct LexedSources<'a> {
    pub source_file_set: &'a SourceFileSet,
    pub token_arena: &'a SourceArena<Token>,
}

impl<'a> LexedSources<'a> {
    pub fn source_range(&self, tokens: &impl Spanned<Token>) -> Option<Range<usize>> {
        match tokens.span() {
            Span::Empty => None,
            Span::Spanning { start, end } => {
                let start = self.token_arena.element(start);
                let end = self.token_arena.element(end);
                Some(start.source_range.start..end.source_range.end)
            }
        }
    }

    pub fn source(&self, tokens: &impl Spanned<Token>) -> &'a str {
        match tokens.span() {
            Span::Empty => "",
            Span::Spanning { start, end } => {
                let source_file_id = self.token_arena.source_file_id(start);
                let start = self.token_arena.element(start);
                let end = self.token_arena.element(end);
                &self.source_file_set.source(source_file_id)
                    [start.source_range.start..end.source_range.end]
            }
        }
    }

    pub fn replacement_suggestion(
        &self,
        tokens: &impl Spanned<Token>,
        replacement: impl Into<String>,
    ) -> Option<ReplacementSuggestion> {
        match tokens.span() {
            Span::Empty => None,
            Span::Spanning { start, end } => {
                let source_file_id = self.token_arena.source_file_id(start);
                let start = self.token_arena.element(start);
                let end = self.token_arena.element(end);
                Some(ReplacementSuggestion {
                    file: source_file_id,
                    span: start.source_range.start..end.source_range.end,
                    replacement: replacement.into(),
                })
            }
        }
    }
}

/// Hacks to enable parsing multi-token operators (`+=`, `>>` etc.)
impl<'a> LexedSources<'a> {
    pub fn span_is_followed_by(&self, tokens: &impl Spanned<Token>, c: char) -> bool {
        match tokens.span() {
            Span::Empty => false,
            Span::Spanning { start, end } => {
                let source_file_id = self.token_arena.source_file_id(start);
                let end = self.token_arena.element(end);
                self.source_file_set.source(source_file_id)[end.source_range.end..].starts_with(c)
            }
        }
    }

    /// Returns whether there is no space between the tokens.
    pub fn tokens_are_hugging_each_other(&self, left: TokenId, right: TokenId) -> bool {
        let left = self.token_arena.element(left);
        let right = self.token_arena.element(right);
        left.source_range.end == right.source_range.start
    }
}

pub struct OwnedSources<'a> {
    pub source_file_set: &'a SourceFileSet,
    pub token_arena: SourceArena<Token>,
}

impl<'a> OwnedSources<'a> {
    pub fn as_borrowed(&self) -> LexedSources<'_> {
        LexedSources {
            source_file_set: self.source_file_set,
            token_arena: &self.token_arena,
        }
    }

    pub fn source_range(&self, tokens: &impl Spanned<Token>) -> Option<Range<usize>> {
        self.as_borrowed().source_range(tokens)
    }

    pub fn source(&self, tokens: &impl Spanned<Token>) -> &'a str {
        // Needs to be copy-pasted from LexedSources' implementation rather than calling
        // .as_borrowed().source() because otherwise the borrow checker sees that the returned string
        // does not live for 'a but for '_.
        match tokens.span() {
            Span::Empty => "",
            Span::Spanning { start, end } => {
                let source_file_id = self.token_arena.source_file_id(start);
                let start = self.token_arena.element(start);
                let end = self.token_arena.element(end);
                &self.source_file_set.source(source_file_id)
                    [start.source_range.start..end.source_range.end]
            }
        }
    }

    pub fn replacement_suggestion(
        &self,
        tokens: &impl Spanned<Token>,
        replacement: impl Into<String>,
    ) -> Option<ReplacementSuggestion> {
        self.as_borrowed()
            .replacement_suggestion(tokens, replacement)
    }
}
