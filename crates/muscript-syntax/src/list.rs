//! Parsing of delimited, comma-separated lists.

use std::marker::PhantomData;

use muscript_foundation::{
    errors::{Diagnostic, Label},
    span::Span,
};
use muscript_lexer::{
    token::TokenKind,
    token_stream::{Channel, TokenStream},
};

use crate::{
    token::{Comma, SingleToken},
    Parse, ParseError, Parser, PredictiveParse,
};

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn parse_greedy_list<E>(&mut self) -> Result<Vec<E>, ParseError>
    where
        E: PredictiveParse,
    {
        let mut elements = vec![];
        loop {
            if self.next_matches::<E>() {
                // NOTE: None of these errors are fatal; if any element happens to not match,
                // we want to continue to report further errors.
                if let Ok(element) = self.parse() {
                    elements.push(element);
                }
            } else {
                break;
            }
        }
        Ok(elements)
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
            let token = self.peek_token();
            match token.kind {
                _ if R::matches(&token, &self.sources) => {
                    self.next_token();
                    break R::default_from_id(token.id);
                }
                TokenKind::EndOfFile => {
                    return Err(TerminatedListError {
                        kind: TerminatedListErrorKind::MissingTerminator,
                        parse: self.make_error(Span::single(token.id)),
                    })
                }
                _ => (),
            }
            if R::KIND.closes().is_some() {
                let result = self.try_with_delimiter_recovery(|parser| parser.parse());
                match result {
                    Ok(node) => elements.push(node),
                    Err(closing) => break closing,
                }
            } else {
                elements.push(
                    self.parse()
                        .map_err(error(TerminatedListErrorKind::Parse))?,
                );
            };
        };

        Ok((elements, terminator))
    }

    pub fn parse_separated_list<E, R, S>(&mut self) -> Result<(Vec<E>, R), SeparatedListError<R>>
    where
        R: SingleToken,
        E: Parse,
        S: SingleToken,
    {
        fn error<R>(
            kind: SeparatedListErrorKind,
        ) -> impl FnOnce(ParseError) -> SeparatedListError<R> {
            move |parse| SeparatedListError {
                kind,
                parse,
                _phantom: PhantomData,
            }
        }

        let mut elements = vec![];
        let close = loop {
            // Will we ever need to peek/next a non-Type context token here?
            // Type was chosen here because of generic lists, which are terminated with `>` and thus
            // `>>` need to be treated as disjoint tokens. But will we ever need a separated list
            // terminated with `>>` (RightShift)?
            let token = self.peek_token_from(Channel::CODE);
            match token.kind {
                TokenKind::EndOfFile => {
                    return Err(SeparatedListError {
                        kind: SeparatedListErrorKind::MissingRight,
                        parse: self.make_error(Span::single(token.id)),
                        _phantom: PhantomData,
                    });
                }
                _ if R::matches(&token, &self.sources) => {
                    // Use default_from_span instead of try_from_token here, since we know the token
                    // is valid. Hopefully this doesn't backfire if at some point we decide that
                    // tokens may store more metadata than just the span.
                    self.next_token_from(Channel::CODE);
                    break R::default_from_id(token.id);
                }
                _ => (),
            }
            // TODO: Have some better error recovery in case parsing the element or any delimiting
            // tokens fails.
            elements.push(self.parse().map_err(error(SeparatedListErrorKind::Parse))?);
            match self.next_token_from(Channel::CODE) {
                token if S::matches(&token, &self.sources) => (),
                token if R::matches(&token, &self.sources) => {
                    // Use default_from_span instead of try_from_token here, since we know the token
                    // is valid. Hopefully this doesn't backfire if at some point we decide that
                    // tokens may store more metadata than just the span.
                    break R::default_from_id(token.id);
                }
                unexpected => {
                    return Err(SeparatedListError {
                        kind: SeparatedListErrorKind::MissingSeparator,
                        parse: self.make_error(Span::single(unexpected.id)),
                        _phantom: PhantomData,
                    });
                }
            }
        };
        Ok((elements, close))
    }

    pub fn parse_comma_separated_list<E, R>(&mut self) -> Result<(Vec<E>, R), SeparatedListError<R>>
    where
        R: SingleToken,
        E: Parse,
    {
        self.parse_separated_list::<E, R, Comma>()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SeparatedListErrorKind {
    Parse,
    MissingRight,
    MissingSeparator,
}

#[derive(Debug, Clone)]
pub struct SeparatedListError<R> {
    pub kind: SeparatedListErrorKind,
    pub parse: ParseError,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Clone, Copy)]
pub enum TerminatedListErrorKind {
    Parse,
    MissingTerminator,
}

#[derive(Debug, Clone)]
pub struct TerminatedListError {
    pub kind: TerminatedListErrorKind,
    pub parse: ParseError,
}

#[derive(Debug, Clone, Copy)]
pub struct SeparatedListDiagnostics<'a> {
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

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn emit_separated_list_diagnostic<L, R>(
        &mut self,
        open: &L,
        error: SeparatedListError<R>,
        diagnostics: SeparatedListDiagnostics<'_>,
    ) -> ParseError
    where
        L: SingleToken,
        R: SingleToken,
    {
        match error.kind {
            SeparatedListErrorKind::Parse => (),
            SeparatedListErrorKind::MissingRight => self.emit_diagnostic(
                Diagnostic::error(diagnostics.missing_right)
                    .with_label(Label::secondary(open, diagnostics.missing_right_label)),
            ),
            SeparatedListErrorKind::MissingSeparator => self.emit_diagnostic(
                Diagnostic::error(diagnostics.missing_comma)
                    .with_label(Label::primary(
                        &error.parse.span,
                        diagnostics.missing_comma_token,
                    ))
                    .with_label(Label::secondary(open, diagnostics.missing_comma_open))
                    .with_note(diagnostics.missing_comma_note),
            ),
        }
        error.parse
    }
}
