use muscript_foundation::{
    errors::{DiagnosticSink, ReplacementSuggestion},
    source::{SourceFileId, SourceFileSet},
    source_arena::SourceArena,
    span::{Span, Spanned},
};
use muscript_lexer::{sources::LexedSources, token::Token};
use muscript_syntax::cst;

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

/// Collection of source files for a class.
#[derive(Debug, Clone)]
pub struct ClassSources {
    pub source_files: Vec<ClassSourceFile>,
}

/// A single class source file.
#[derive(Debug, Clone)]
pub struct ClassSourceFile {
    pub id: SourceFileId,
    pub parsed: cst::File,
}

/// External source providing source code for classes.
pub trait CompilerInput {
    /// Returns whether a class with the given name exists.
    fn class_exists(&self, class_name: &str) -> bool;

    /// Returns the source file IDs of a class.
    ///
    /// In case `None` is returned, a class with the given name does not exist.
    fn class_source_ids(&self, class_name: &str) -> Option<Vec<SourceFileId>>;

    /// Returns the parsed sources of a class. May be empty if any errors occur.
    ///
    /// In case `None` is returned, a class with the given name does not exist.
    ///
    /// `diagnostics` should be filled in with any errors that occur during parsing. Files that
    /// irrecoverably fail to parse should not be included in the output.
    fn parsed_class_sources(
        &self,
        sources: &mut OwnedSources<'_>,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> Option<ClassSources>;
}
