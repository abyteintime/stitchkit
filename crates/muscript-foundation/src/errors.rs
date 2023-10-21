//! Types for error reporting.
//!
//! The error reporting in MuScript is largely inspired by the Rust compiler, though arguably it's a
//! lot simpler.

mod sink;

use std::ops::Range;

use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};

use crate::{
    source::{SourceFileId, SourceFileSet},
    source_arena::SourceArena,
    span::{Span, Spanned},
};

pub use sink::*;

/// Trait for types which span across a range of source characters.
pub trait SourceRange {
    fn source_range(&self) -> Range<usize>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LabelStyle {
    /// Labels that describe the primary cause of a diagnostic.
    Primary,
    /// Labels that provide additional context for a diagnostic.
    Secondary,
}

/// Labels allow you to attach information about where in the code an error occurred.
#[derive(Debug)]
pub struct Label<T> {
    /// The style of the label; `Primary` should be used for the crux of the problem, and
    /// `Secondary` may be used for extra annotations shown alongside primary labels.
    pub style: LabelStyle,
    /// The span this label labels.
    pub span: Span<T>,
    /// The message attached to the label.
    pub message: String,
}

impl<T> Clone for Label<T> {
    fn clone(&self) -> Self {
        Self {
            style: self.style,
            span: self.span,
            message: self.message.clone(),
        }
    }
}

impl<T> Label<T> {
    /// Create a label passing the style as an argument.
    ///
    /// You should generally prefer the helper functions [`Label::primary`] and [`Label::secondary`]
    /// instead of this.
    pub fn new<O, M>(style: LabelStyle, span: &impl Spanned<T>, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        let message = message.into();
        let message = message.map(|x| x.into());
        Self {
            style,
            span: span.span(),
            message: message.unwrap_or_default(),
        }
    }

    /// Creates a primary label placed at the given span, with the given message.
    pub fn primary<O, M>(span: &impl Spanned<T>, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Primary, span, message)
    }

    /// Creates a secondary label placed at the given span, with the given message.
    pub fn secondary<O, M>(span: &impl Spanned<T>, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Secondary, span, message)
    }
}

/// Suggestion for what to replace a span with that might make the diagnostic go away.
#[derive(Debug, Clone)]
pub struct ReplacementSuggestion {
    /// The file within which the replacement should be done.
    pub file: SourceFileId,
    /// The span of bytes to replace.
    pub span: Range<usize>,
    /// The replacement string.
    pub replacement: String,
}

/// The type of a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    Normal,
    Debug,
}

/// A note attached to the bottom of the diagnostic, providing additional help or context about the
/// error.
#[derive(Debug, Clone)]
pub struct Note {
    /// This note's kind.
    pub kind: NoteKind,
    /// The error text.
    pub text: String,
    /// An optional replacement suggestion, which will be displayed alongside the note.
    pub suggestion: Option<ReplacementSuggestion>,
}

impl From<String> for Note {
    fn from(text: String) -> Self {
        Self {
            kind: NoteKind::Normal,
            text,
            suggestion: None,
        }
    }
}

impl From<(String, ReplacementSuggestion)> for Note {
    fn from((text, suggestion): (String, ReplacementSuggestion)) -> Self {
        Self {
            kind: NoteKind::Normal,
            text,
            suggestion: Some(suggestion),
        }
    }
}

impl From<(String, Option<ReplacementSuggestion>)> for Note {
    fn from((text, suggestion): (String, Option<ReplacementSuggestion>)) -> Self {
        Self {
            kind: NoteKind::Normal,
            text,
            suggestion,
        }
    }
}

impl From<&str> for Note {
    fn from(text: &str) -> Self {
        Self::from(text.to_string())
    }
}

impl From<(&str, ReplacementSuggestion)> for Note {
    fn from((text, suggestion): (&str, ReplacementSuggestion)) -> Self {
        Self::from((text.to_string(), suggestion))
    }
}

impl From<(&str, Option<ReplacementSuggestion>)> for Note {
    fn from((text, suggestion): (&str, Option<ReplacementSuggestion>)) -> Self {
        Self::from((text.to_string(), suggestion))
    }
}

/// Diagnostic severity.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Severity {
    /// A help message.
    Help,
    /// A note.
    Note,
    /// A warning.
    Warning,
    /// An error.
    Error,
    /// An unexpected bug.
    Bug,
}

/// Diagnostic describing a problem encountered within the code.
#[derive(Debug)]
pub struct Diagnostic<T> {
    /// The diagnostic's severity.
    pub severity: Severity,
    /// The diagnostic's error code.
    pub code: Option<String>,
    /// The message describing the issue.
    pub message: String,
    /// Labels attached to the diagnostic.
    pub labels: Vec<Label<T>>,
    /// Additional notes providing context.
    pub notes: Vec<Note>,
    /// Diagnostics providing additional context on this diagnostic.
    pub children: Vec<Diagnostic<T>>,
}

impl<T> Clone for Diagnostic<T> {
    fn clone(&self) -> Self {
        Self {
            severity: self.severity,
            code: self.code.clone(),
            message: self.message.clone(),
            labels: self.labels.clone(),
            notes: self.notes.clone(),
            children: self.children.clone(),
        }
    }
}

impl<T> Diagnostic<T> {
    /// Creates a new diagnostic with the severity passed in as an argument. You should generally
    /// prefer the convenience functions over this:
    /// - [`Diagnostic::bug`]
    /// - [`Diagnostic::error`]
    /// - [`Diagnostic::warning`]
    /// - [`Diagnostic::note`]
    /// - [`Diagnostic::help`]
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: None,
            message: message.into(),
            labels: vec![],
            notes: vec![],
            children: vec![],
        }
    }

    /// Creates a new bug-level diagnostic.
    ///
    /// Note that unlike other severities, since this may be triggered by an actual bug
    /// (ie. an unhandled external error,) the message passed in may be anything that can be
    /// [`Display`][std::fmt::Display]ed as text.
    pub fn bug(error: impl ToString) -> Self {
        Self::new(Severity::Bug, error.to_string())
    }

    /// Creates a new error-level diagnostic with the given message.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    /// Creates a new warning-level diagnostic with the given message.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    /// Creates a new note-level diagnostic with the given message.
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    /// Creates a new help-level diagnostic with the given message.
    pub fn help(message: impl Into<String>) -> Self {
        Self::new(Severity::Help, message)
    }

    /// Sets the diagnostic's error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Adds a label to the diagnostic.
    pub fn with_label(mut self, label: Label<T>) -> Self {
        self.labels.push(label);
        self
    }

    /// Adds a note to the diagnostic.
    pub fn with_note(mut self, note: impl Into<Note>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds an optional note to the diagnostic.
    pub fn with_optional_note(mut self, note: impl Into<Option<Note>>) -> Self {
        if let Some(note) = note.into() {
            self.notes.push(note);
        }
        self
    }

    /// Adds a child to the diagnostic.
    pub fn with_child(mut self, child: Diagnostic<T>) -> Self {
        self.children.push(child);
        self
    }
}

impl<T> Diagnostic<T>
where
    T: SourceRange,
{
    /// Emits the diagnostic to standard error.
    pub fn emit_to_stderr(
        &self,
        files: &SourceFileSet,
        source_arena: &SourceArena<T>,
        config: &DiagnosticConfig,
    ) -> Result<(), codespan_reporting::files::Error> {
        term::emit(
            &mut StandardStream::stderr(ColorChoice::Auto),
            &term::Config::default(),
            files,
            &self.to_codespan(source_arena, config),
        )?;
        for child in &self.children {
            child.emit_to_stderr(files, source_arena, config)?;
        }
        Ok(())
    }

    pub fn to_codespan(
        &self,
        source_arena: &SourceArena<T>,
        config: &DiagnosticConfig,
    ) -> codespan_reporting::diagnostic::Diagnostic<SourceFileId> {
        codespan_reporting::diagnostic::Diagnostic {
            severity: match self.severity {
                Severity::Help => codespan_reporting::diagnostic::Severity::Help,
                Severity::Note => codespan_reporting::diagnostic::Severity::Note,
                Severity::Warning => codespan_reporting::diagnostic::Severity::Warning,
                Severity::Error => codespan_reporting::diagnostic::Severity::Error,
                Severity::Bug => codespan_reporting::diagnostic::Severity::Bug,
            },
            code: self.code.clone(),
            message: self.message.clone(),
            labels: self
                .labels
                .iter()
                .filter_map(|label| match label.span {
                    Span::Empty => None,
                    Span::Spanning { start, end } => Some(codespan_reporting::diagnostic::Label {
                        style: match label.style {
                            LabelStyle::Primary => {
                                codespan_reporting::diagnostic::LabelStyle::Primary
                            }
                            LabelStyle::Secondary => {
                                codespan_reporting::diagnostic::LabelStyle::Secondary
                            }
                        },
                        file_id: source_arena.source_file_id(start),
                        range: {
                            let start_range = source_arena.element(start).source_range();
                            let end_range = source_arena.element(end).source_range();
                            start_range.start.min(end_range.start)
                                ..start_range.end.max(end_range.end)
                        },
                        message: label.message.clone(),
                    }),
                })
                .collect(),
            notes: self
                .notes
                .iter()
                .filter(|&note| note.kind != NoteKind::Debug || config.show_debug_info)
                .map(|note| {
                    if let Some(sug) = note.suggestion.clone() {
                        format!("{}: `{}`", note.text, sug.replacement)
                    } else {
                        note.text.clone()
                    }
                })
                .collect(),
        }
    }
}

pub struct DiagnosticConfig {
    pub show_debug_info: bool,
}
