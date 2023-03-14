//! Types for error reporting.
//!
//! The error reporting in MuScript is largely inspired by the Rust compiler, though arguably it's a
//! lot simpler.

use codespan_reporting::{
    term,
    term::termcolor::{ColorChoice, StandardStream},
};

use crate::source::{SourceFileId, SourceFileSet, Span};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LabelStyle {
    /// Labels that describe the primary cause of a diagnostic.
    Primary,
    /// Labels that provide additional context for a diagnostic.
    Secondary,
}

/// Labels allow you to attach information about where in the code an error occurred.
#[derive(Debug, Clone)]
pub struct Label {
    /// The style of the label; `Primary` should be used for the crux of the problem, and
    /// `Secondary` may be used for extra annotations shown alongside primary labels.
    pub style: LabelStyle,
    /// The span this label labels.
    pub span: Span,
    /// The message attached to the label.
    pub message: String,
    /// The file in which the label should appear. If [`None`], the file is inherited from the
    /// enclosing [`Diagnostic`].
    pub file: Option<SourceFileId>,
}

impl Label {
    /// Create a label passing the style as an argument.
    ///
    /// You should generally prefer the helper functions [`Label::primary`] and [`Label::secondary`]
    /// instead of this.
    pub fn new<O, M>(style: LabelStyle, span: Span, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        let message = message.into();
        let message = message.map(|x| x.into());
        Self {
            style,
            span,
            message: message.unwrap_or_default(),
            file: None,
        }
    }

    /// Creates a primary label placed at the given span, with the given message.
    pub fn primary<O, M>(span: Span, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Primary, span, message)
    }

    /// Creates a secondary label placed at the given span, with the given message.
    pub fn secondary<O, M>(span: Span, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Secondary, span, message)
    }

    /// Specifies an explicit file to use for this label. By default, the enclosing diagnostic's
    /// file is used.
    pub fn in_file(mut self, file: SourceFileId) -> Self {
        self.file = Some(file);
        self
    }
}

/// Suggestion for what to replace a span with that might make the diagnostic go away.
#[derive(Debug, Clone)]
pub struct ReplacementSuggestion {
    /// The span of bytes to replace.
    pub span: Span,
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
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The diagnostic's severity.
    pub severity: Severity,
    /// The diagnostic's error code.
    pub code: Option<String>,
    /// The message describing the issue.
    pub message: String,
    /// The source file within which the issue occurred.
    pub source_file: SourceFileId,
    /// Labels attached to the diagnostic.
    pub labels: Vec<Label>,
    /// Additional notes providing context.
    pub notes: Vec<Note>,
    /// Diagnostics providing additional context on this diagnostic.
    pub children: Vec<Diagnostic>,
}

impl Diagnostic {
    /// Creates a new diagnostic with the severity passed in as an argument. You should generally
    /// prefer the convenience functions over this:
    /// - [`Diagnostic::bug`]
    /// - [`Diagnostic::error`]
    /// - [`Diagnostic::warning`]
    /// - [`Diagnostic::note`]
    /// - [`Diagnostic::help`]
    pub fn new(severity: Severity, source_file: SourceFileId, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: None,
            message: message.into(),
            source_file,
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
    pub fn bug(file: SourceFileId, error: impl ToString) -> Self {
        Self::new(Severity::Bug, file, error.to_string())
    }

    /// Creates a new error-level diagnostic with the given message.
    pub fn error(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, file, message)
    }

    /// Creates a new warning-level diagnostic with the given message.
    pub fn warning(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, file, message)
    }

    /// Creates a new note-level diagnostic with the given message.
    pub fn note(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, file, message)
    }

    /// Creates a new help-level diagnostic with the given message.
    pub fn help(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Help, file, message)
    }

    /// Sets the diagnostic's error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Adds a label to the diagnostic.
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Adds a note to the diagnostic.
    pub fn with_note(mut self, note: impl Into<Note>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds a child to the diagnostic.
    pub fn with_child(mut self, child: Diagnostic) -> Self {
        self.children.push(child);
        self
    }

    /// Emits the diagnostic to standard error.
    pub fn emit_to_stderr(
        &self,
        files: &SourceFileSet,
        config: &DiagnosticConfig,
    ) -> Result<(), codespan_reporting::files::Error> {
        term::emit(
            &mut StandardStream::stderr(ColorChoice::Auto),
            &term::Config::default(),
            files,
            &self.to_codespan(config),
        )?;
        for child in &self.children {
            child.emit_to_stderr(files, config)?;
        }
        Ok(())
    }

    pub fn to_codespan(
        &self,
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
                .map(|label| codespan_reporting::diagnostic::Label {
                    style: match label.style {
                        LabelStyle::Primary => codespan_reporting::diagnostic::LabelStyle::Primary,
                        LabelStyle::Secondary => {
                            codespan_reporting::diagnostic::LabelStyle::Secondary
                        }
                    },
                    file_id: label.file.unwrap_or(self.source_file),
                    range: label.span.to_usize_range(),
                    message: label.message.clone(),
                })
                .collect(),
            notes: self
                .notes
                .iter()
                .filter_map(|note| {
                    (note.kind != NoteKind::Debug || config.show_debug_info).then(|| {
                        if let Some(sug) = note.suggestion.clone() {
                            format!("{}: `{}`", note.text, sug.replacement)
                        } else {
                            note.text.clone()
                        }
                    })
                })
                .collect(),
        }
    }
}

pub struct DiagnosticConfig {
    pub show_debug_info: bool,
}
