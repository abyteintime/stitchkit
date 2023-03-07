mod recovery;

use muscript_foundation::{
    errors::{Diagnostic, Label, Note, NoteKind},
    source::{SourceFileId, Span},
};

use crate::lexis::{
    token::{SingleToken, Token, TokenKindMismatch},
    LexError, TokenStream,
};

pub use recovery::*;

pub struct Parser<'a, T> {
    pub file: SourceFileId,
    pub input: &'a str,
    pub tokens: T,
    errors: Vec<Diagnostic>,
    // TODO: This should probably be a &mut because with how it's done currently creating a sub
    // parser involves cloning the vector.
    rule_traceback: Vec<&'static str>,
}

impl<'a, T> Parser<'a, T> {
    pub fn new(file: SourceFileId, input: &'a str, tokens: T) -> Self {
        Self {
            file,
            input,
            tokens,
            errors: vec![],
            rule_traceback: Vec::with_capacity(32),
        }
    }

    pub fn sub(&mut self) -> Parser<'a, &mut T> {
        Parser {
            file: self.file,
            input: self.input,
            tokens: &mut self.tokens,
            errors: vec![],
            rule_traceback: self.rule_traceback.clone(),
        }
    }

    pub fn errors(&self) -> &[Diagnostic] {
        &self.errors
    }

    pub fn into_errors(self) -> Vec<Diagnostic> {
        self.errors
    }
}

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn bail<TT>(&mut self, error_span: Span, error: Diagnostic) -> Result<TT, ParseError> {
        self.emit_diagnostic(error);
        Err(ParseError::new(error_span))
    }

    pub fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        let diagnostic = diagnostic.with_note(Note {
            kind: NoteKind::Debug,
            text: {
                let mut s = String::from("parser traceback (innermost rule last):");
                for rule in &self.rule_traceback {
                    s.push_str("\n  - ");
                    s.push_str(rule);
                }
                s
            },
            suggestion: None,
        });
        let diagnostic = self.tokens.contextualize_diagnostic(diagnostic);
        self.errors.push(diagnostic);
    }
}

impl<'a, T> Parser<'a, T>
where
    T: ParseStream,
{
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        self.tokens
            .next()
            .map_err(|LexError { span, diagnostics }| {
                for diagnostic in diagnostics {
                    self.emit_diagnostic(diagnostic);
                }
                ParseError { span }
            })
    }

    pub fn peek_token(&mut self) -> Result<Token, ParseError> {
        self.tokens
            .peek()
            .map_err(|LexError { span, .. }| ParseError { span })
    }

    pub fn expect_token<Tok>(&mut self) -> Result<Tok, ParseError>
    where
        Tok: SingleToken,
    {
        match self.next_token() {
            Ok(token) => {
                let input = token.span.get_input(self.input);
                Tok::try_from_token(token, input).map_err(|TokenKindMismatch(token)| {
                    self.emit_diagnostic(
                        Diagnostic::error(self.file, format!("{} expected", Tok::NAME)).with_label(
                            Label::primary(token.span(), format!("{} expected here", Tok::NAME)),
                        ),
                    );
                    ParseError::new(token.span())
                })
            }
            Err(ParseError { span }) => {
                // Try to recover from the lexis error and keep on parsing beyond this point.
                // We fabricate the token from the reported span.
                Ok(Tok::default_from_span(span))
            }
        }
    }

    pub fn parse<N>(&mut self) -> Result<N, ParseError>
    where
        N: Parse,
    {
        self.rule_traceback.push(std::any::type_name::<N>());
        #[allow(deprecated)]
        let result = N::parse(self);
        self.rule_traceback.pop();
        result
    }

    pub fn parse_with_error<N>(
        &mut self,
        diagnostic: impl FnOnce(&Self, Span) -> Diagnostic,
    ) -> Result<N, ParseError>
    where
        N: Parse,
    {
        self.sub().parse().map_err(|error| {
            self.emit_diagnostic(diagnostic(self, error.span));
            error
        })
    }
}

/// The AST node could not be parsed.
#[derive(Debug, Clone, Copy)]
pub struct ParseError {
    pub span: Span,
}

impl ParseError {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

/// Token stream which can provide data for error recovery.
pub trait ParseStream: TokenStream {
    fn nesting_level(&self) -> usize;
}

impl<T> ParseStream for &mut T
where
    T: ParseStream,
{
    fn nesting_level(&self) -> usize {
        <T as ParseStream>::nesting_level(self)
    }
}

pub trait Parse: Sized {
    /// NOTE: This is deprecated because it should not be used directly, as it doesn't do any extra
    /// processing or error recovery.
    /// You generally want to use [`Parser::parse`] instead of this.
    #[deprecated(note = "use [`Parser::parse`] instead of this")]
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError>;
}

pub trait PredictiveParse: Parse {
    /// Returns `true` if this syntactic construct starts with the given token.
    fn started_by(token: &Token, input: &str) -> bool;
}

impl<N> Parse for Option<N>
where
    N: PredictiveParse,
{
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        if let Ok(next_token) = parser.peek_token() {
            if N::started_by(&next_token, parser.input) {
                #[allow(deprecated)]
                Ok(Some(parser.parse()?))
            } else {
                Ok(None)
            }
        } else {
            // It's fine if there's a lexing error; it'll be taken care of by whatever _requires_
            // the following token.
            Ok(None)
        }
    }
}
