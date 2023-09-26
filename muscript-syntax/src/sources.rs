use std::collections::HashMap;

use muscript_foundation::{
    errors::{Diagnostic, ReplacementSuggestion},
    source::SourceFileSet,
    source_arena::SourceArena,
    span::{Span, Spanned},
};

use crate::lexis::token::{Token, TokenId};

#[derive(Clone, Copy)]
pub struct LexedSources<'a> {
    pub source_file_set: &'a SourceFileSet,
    pub token_arena: &'a SourceArena<Token>,
    pub errors: &'a HashMap<TokenId, Diagnostic<Token>>,
}

impl<'a> LexedSources<'a> {
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

/// Hacks to enable parsing compound assignments (`+=` etc.)
impl<'a> LexedSources<'a> {
    pub(crate) fn span_is_followed_by(&self, tokens: &impl Spanned<Token>, c: char) -> bool {
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
    pub(crate) fn tokens_are_hugging_each_other(&self, left: TokenId, right: TokenId) -> bool {
        let left = self.token_arena.element(left);
        let right = self.token_arena.element(right);
        left.source_range.end == right.source_range.start
    }
}
