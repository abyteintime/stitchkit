use std::ops::Range;

pub use codespan_reporting::diagnostic::LabelStyle;
pub use codespan_reporting::diagnostic::Severity;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;

use crate::source::SourceFileId;
use crate::source::SourceFileSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span(pub Range<usize>);

pub struct Label {
    pub style: LabelStyle,
    pub span: Span,
    pub message: String,
    pub file: Option<SourceFileId>,
}

impl Label {
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

    pub fn primary<O, M>(span: Span, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Primary, span, message)
    }

    pub fn secondary<O, M>(span: Span, message: O) -> Self
    where
        O: Into<Option<M>>,
        M: Into<String>,
    {
        Self::new(LabelStyle::Secondary, span, message)
    }

    pub fn in_file(mut self, file: SourceFileId) -> Self {
        self.file = Some(file);
        self
    }
}

pub struct ReplacementSuggestion {
    pub span: Span,
    pub replacement: String,
}

pub struct Note {
    pub text: String,
    pub suggestion: Option<ReplacementSuggestion>,
}

impl From<String> for Note {
    fn from(text: String) -> Self {
        Self {
            text,
            suggestion: None,
        }
    }
}

impl From<(String, ReplacementSuggestion)> for Note {
    fn from((text, suggestion): (String, ReplacementSuggestion)) -> Self {
        Self {
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

pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub source_file: SourceFileId,
    pub labels: Vec<Label>,
    pub notes: Vec<Note>,
}

impl Diagnostic {
    pub fn new(severity: Severity, source_file: SourceFileId, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: None,
            message: message.into(),
            source_file,
            labels: vec![],
            notes: vec![],
        }
    }

    pub fn bug(file: SourceFileId, error: impl ToString) -> Self {
        Self::new(Severity::Bug, file, error.to_string())
    }

    pub fn error(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, file, message)
    }

    pub fn warning(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, file, message)
    }

    pub fn note(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, file, message)
    }

    pub fn help(file: SourceFileId, message: impl Into<String>) -> Self {
        Self::new(Severity::Help, file, message)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_note(mut self, note: impl Into<Note>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn emit_to_stderr(
        self,
        files: &SourceFileSet,
    ) -> Result<(), codespan_reporting::files::Error> {
        term::emit(
            &mut StandardStream::stderr(ColorChoice::Auto),
            &term::Config::default(),
            files,
            &self.into(),
        )
    }
}

impl From<Diagnostic> for codespan_reporting::diagnostic::Diagnostic<SourceFileId> {
    fn from(diag: Diagnostic) -> Self {
        Self {
            severity: diag.severity,
            code: diag.code,
            message: diag.message,
            labels: diag
                .labels
                .into_iter()
                .map(|label| codespan_reporting::diagnostic::Label {
                    style: label.style,
                    file_id: label.file.unwrap_or(diag.source_file),
                    range: label.span.0,
                    message: label.message,
                })
                .collect(),
            notes: diag
                .notes
                .into_iter()
                .map(|note| {
                    if let Some(sug) = note.suggestion {
                        format!("{}: `{}`", note.text, sug.replacement)
                    } else {
                        note.text
                    }
                })
                .collect(),
        }
    }
}
