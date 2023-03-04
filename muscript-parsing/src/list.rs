//! Parsing of delimited, comma-separated lists.

use std::marker::PhantomData;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{
        token::{SingleToken, Token, TokenKind},
        TokenStream,
    },
    parsing::{Parse, ParseError, Parser},
};

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn parse_delimited_list<E, R>(&mut self) -> Result<(Vec<E>, R), DelimitedListError<R>>
    where
        R: SingleToken,
        E: Parse,
    {
        fn error<R>(
            kind: DelimitedListErrorKind,
        ) -> impl FnOnce(ParseError) -> DelimitedListError<R> {
            move |parse| DelimitedListError {
                kind,
                parse,
                _phantom: PhantomData,
            }
        }

        let mut elements = vec![];
        let close = loop {
            let token = self
                .peek_token()
                .map_err(error(DelimitedListErrorKind::Parse))?;
            match token.kind {
                TokenKind::EndOfFile => {
                    return Err(DelimitedListError {
                        kind: DelimitedListErrorKind::MissingRight,
                        parse: ParseError::new(token.span),
                        _phantom: PhantomData,
                    });
                }
                _ if R::matches(&token, token.span.get_input(self.input)) => {
                    // Use default_from_span instead of try_from_token here, since we know the token
                    // is valid. Hopefully this doesn't backfire if at some point we decide that
                    // tokens may store more metadata than just the span.
                    self.next_token()
                        .map_err(error(DelimitedListErrorKind::Parse))?;
                    break R::default_from_span(token.span);
                }
                _ => (),
            }
            // TODO: Have some better error recovery in case parsing the element or any delimiting
            // tokens fails.
            elements.push(self.parse().map_err(error(DelimitedListErrorKind::Parse))?);
            match self
                .next_token()
                .map_err(error(DelimitedListErrorKind::Parse))?
            {
                Token {
                    span: _, // TODO: Maybe save the span info? How useful would it be?
                    kind: TokenKind::Comma,
                } => (),
                token if R::matches(&token, token.span.get_input(self.input)) => {
                    // Use default_from_span instead of try_from_token here, since we know the token
                    // is valid. Hopefully this doesn't backfire if at some point we decide that
                    // tokens may store more metadata than just the span.
                    break R::default_from_span(token.span);
                }
                unexpected => {
                    return Err(DelimitedListError {
                        kind: DelimitedListErrorKind::MissingComma,
                        parse: ParseError::new(unexpected.span),
                        _phantom: PhantomData,
                    });
                }
            }
        };
        Ok((elements, close))
    }

    pub fn parse_terminated_list<E, R>(&mut self) -> Result<(Vec<E>, R), TerminatedListError>
    where
        E: Parse,
        R: SingleToken,
    {
        fn error(kind: TerminatedListErrorKind) -> impl FnOnce(ParseError) -> TerminatedListError {
            move |parse| TerminatedListError { kind, parse }
        }

        let mut elements = vec![];
        let terminator = loop {
            let token = self
                .peek_token()
                .map_err(error(TerminatedListErrorKind::Parse))?;
            match token.kind {
                _ if R::matches(&token, self.input) => {
                    self.next_token().expect("the token was already parsed");
                    break R::default_from_span(token.span);
                }
                TokenKind::EndOfFile => {
                    return Err(TerminatedListError {
                        kind: TerminatedListErrorKind::MissingTerminator,
                        parse: ParseError::new(token.span),
                    })
                }
                _ => (),
            }
            // NOTE: Error recovery here is not really possible since we don't have an anchor point
            // to eat tokens until. As such it is up to the individual elements to recover from
            // parse errors.
            elements.push(
                self.parse()
                    .map_err(error(TerminatedListErrorKind::Parse))?,
            );
        };

        Ok((elements, terminator))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DelimitedListErrorKind {
    Parse,
    MissingRight,
    MissingComma,
}

#[derive(Debug, Clone, Copy)]
pub struct DelimitedListError<R> {
    pub kind: DelimitedListErrorKind,
    pub parse: ParseError,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Clone, Copy)]
pub enum TerminatedListErrorKind {
    Parse,
    MissingTerminator,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminatedListError {
    pub kind: TerminatedListErrorKind,
    pub parse: ParseError,
}

#[derive(Debug, Clone, Copy)]
pub struct DelimitedListDiagnostics<'a> {
    /// ```text
    /// missing `right` to close [thing]
    /// ```
    pub missing_right: &'a str,
    /// ```text
    /// this `left` does not have a matching `right`
    /// ```
    pub missing_right_label: &'a str,

    /// ```text
    /// `,` or `right` expected after [thing]
    /// ```
    pub missing_comma: &'a str,
    /// ```text
    /// the [thing] starts here
    /// ```
    pub missing_comma_open: &'a str,
    /// ```text
    /// this was expected to continue or close the [thing]
    /// ```
    pub missing_comma_token: &'a str,
    /// ```text
    /// note: [elements] must be separated by commas `,`
    /// ```
    pub missing_comma_note: &'a str,
}

impl<'a, T> Parser<'a, T> {
    pub fn emit_delimited_list_diagnostic<L, R>(
        &mut self,
        open: &L,
        error: DelimitedListError<R>,
        diagnostics: DelimitedListDiagnostics<'_>,
    ) -> ParseError
    where
        L: SingleToken,
        R: SingleToken,
    {
        match error.kind {
            DelimitedListErrorKind::Parse => (),
            DelimitedListErrorKind::MissingRight => self.emit_diagnostic(
                Diagnostic::error(self.file, diagnostics.missing_right).with_label(
                    Label::secondary(open.span(), diagnostics.missing_right_label),
                ),
            ),
            DelimitedListErrorKind::MissingComma => self.emit_diagnostic(
                Diagnostic::error(self.file, diagnostics.missing_comma)
                    .with_label(Label::primary(
                        error.parse.span,
                        diagnostics.missing_comma_token,
                    ))
                    .with_label(Label::secondary(
                        open.span(),
                        diagnostics.missing_comma_open,
                    ))
                    .with_note(diagnostics.missing_comma_note),
            ),
        }
        error.parse
    }
}
