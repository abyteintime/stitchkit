use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{SourceFileId, Span},
};

use crate::lexis::{
    token::{SingleToken, Token, TokenKindMismatch},
    LexError, TokenStream,
};

pub struct Parser<'a, T> {
    pub file: SourceFileId,
    pub input: &'a str,
    pub tokens: T,
    pub errors: Vec<Diagnostic>,
}

impl<'a, T> Parser<'a, T> {
    pub fn new(file: SourceFileId, input: &'a str, tokens: T) -> Self {
        Self {
            file,
            input,
            tokens,
            errors: vec![],
        }
    }

    pub fn sub(&mut self) -> Parser<'a, &mut T> {
        Parser {
            file: self.file,
            input: self.input,
            tokens: &mut self.tokens,
            errors: vec![],
        }
    }

    pub fn bail<TT>(&mut self, error_span: Span, error: Diagnostic) -> Result<TT, ParseError> {
        self.errors.push(error);
        Err(ParseError::new(error_span))
    }
}

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn next_token(&mut self) -> Result<Token, Span> {
        self.tokens.next().map_err(|LexError { span, diagnostic }| {
            self.errors.push(*diagnostic);
            span
        })
    }

    pub fn peek_token(&mut self) -> Result<Token, Span> {
        self.tokens.peek().map_err(|LexError { span, .. }| span)
    }

    pub fn expect_token<Tok>(&mut self) -> Result<Tok, ParseError>
    where
        Tok: SingleToken,
    {
        match self.next_token() {
            Ok(token) => {
                let input = token.span.get_input(self.input);
                Tok::try_from_token(token, input).map_err(|TokenKindMismatch(token)| {
                    self.errors.push(
                        Diagnostic::error(self.file, format!("{} expected", Tok::NAME)).with_label(
                            Label::primary(token.span(), format!("{} expected here", Tok::NAME)),
                        ),
                    );
                    ParseError::new(token.span())
                })
            }
            Err(span) => {
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
        N::parse(self)
    }

    pub fn parse_with_error<N>(
        &mut self,
        diagnostic: impl FnOnce(&Self, Span) -> Diagnostic,
    ) -> Result<N, ParseError>
    where
        N: Parse,
    {
        self.sub().parse().map_err(|error| {
            self.errors.push(diagnostic(self, error.span));
            error
        })
    }
}

/// The AST node could not be parsed.
pub struct ParseError {
    pub span: Span,
}

impl ParseError {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

pub trait Parse: Sized {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError>;
}

pub trait PredictiveParse: Parse {
    /// Returns `true` if this syntactic construct starts with the given token.
    fn starts_with(token: &Token, input: &str) -> bool;
}

impl<N> Parse for Option<N>
where
    N: PredictiveParse,
{
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        if let Ok(next_token) = parser.peek_token() {
            if N::starts_with(&next_token, parser.input) {
                Ok(Some(N::parse(parser)?))
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
