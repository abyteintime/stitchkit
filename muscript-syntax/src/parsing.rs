mod lazy;
mod recovery;

use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label, Note, NoteKind};
use tracing::warn;

use crate::{
    lexis::{
        token::{AnyToken, SingleToken, Token, TokenSpan},
        Channel, LexicalContext, TokenStream,
    },
    sources::LexedSources,
};

pub use lazy::*;
pub use recovery::*;

pub struct Parser<'a, T> {
    pub sources: LexedSources<'a>,
    pub tokens: T,
    diagnostics: &'a mut dyn DiagnosticSink<Token>,
    // TODO: This should probably be a &mut because with how it's done currently creating a sub
    // parser involves cloning the vector.
    #[cfg(feature = "parse-traceback")]
    rule_traceback: Vec<&'static str>,
}

impl<'a, T> Parser<'a, T> {
    pub fn new(
        sources: LexedSources<'a>,
        tokens: T,
        diagnostics: &'a mut dyn DiagnosticSink<Token>,
    ) -> Self {
        Self {
            sources,
            tokens,
            diagnostics,
            #[cfg(feature = "parse-traceback")]
            rule_traceback: Vec::with_capacity(32),
        }
    }

    pub fn sub<'b>(
        &'b mut self,
        diagnostics: Option<&'b mut dyn DiagnosticSink<Token>>,
    ) -> Parser<'b, &mut T> {
        Parser {
            sources: self.sources,
            tokens: &mut self.tokens,
            diagnostics: diagnostics
                .map(|r| {
                    let d: &mut dyn DiagnosticSink<Token> = r;
                    d
                })
                .unwrap_or(self.diagnostics),
            #[cfg(feature = "parse-traceback")]
            rule_traceback: self.rule_traceback.clone(),
        }
    }
}

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    fn rule_traceback(&self) -> Vec<&'static str> {
        #[cfg(feature = "parse-traceback")]
        {
            self.rule_traceback.clone()
        }
        #[cfg(not(feature = "parse-traceback"))]
        {
            vec![]
        }
    }

    pub fn make_error(&self, span: TokenSpan) -> ParseError {
        ParseError::new(span, self.rule_traceback())
    }

    pub fn bail<TT>(
        &mut self,
        error_span: TokenSpan,
        error: Diagnostic<Token>,
    ) -> Result<TT, ParseError> {
        self.emit_diagnostic(error);
        Err(self.make_error(error_span))
    }

    pub fn emit_diagnostic(&mut self, diagnostic: Diagnostic<Token>) {
        #[cfg(feature = "parse-traceback")]
        let diagnostic = diagnostic.with_note(Note {
            kind: NoteKind::Debug,
            text: {
                let mut s = String::from("parser traceback (innermost rule last):");
                for rule in &self.rule_traceback {
                    s.push_str("\n    ");
                    s.push_str(rule);
                }
                s
            },
            suggestion: None,
        });
        let diagnostic = self.tokens.contextualize_diagnostic(diagnostic);
        self.diagnostics.emit(diagnostic);
    }
}

impl<'a, T> Parser<'a, T>
where
    T: ParseStream,
{
    pub fn next_token_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken {
        loop {
            let token = self.tokens.next_from(context, channel);
            if token.kind.channel() == Channel::ERROR && !channel.contains(Channel::ERROR) {
                if let Some(diagnostic) = self.sources.errors.get(&token.id) {
                    self.diagnostics.emit(diagnostic.clone());
                } else {
                    warn!(?token, "error token without corresponding diagnostic");
                }
            } else {
                return token;
            }
        }
    }

    pub fn next_token(&mut self) -> AnyToken {
        self.next_token_from(LexicalContext::Default, Channel::CODE)
    }

    pub fn peek_token_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken {
        self.tokens.peek_from(context, channel)
    }

    pub fn peek_token(&mut self) -> AnyToken {
        self.peek_token_from(LexicalContext::Default, Channel::CODE)
    }

    pub fn expect_token_from<Tok>(
        &mut self,
        context: LexicalContext,
        channel: Channel,
    ) -> Result<Tok, ParseError>
    where
        Tok: SingleToken,
    {
        let token = self.next_token_from(context, channel | Tok::LISTEN_TO_CHANNELS);
        Tok::try_from_token(token, &self.sources).map_err(|error| {
            self.emit_diagnostic(
                Diagnostic::error(format!("{} expected", Tok::NAME))
                    .with_label(Label::primary(
                        &TokenSpan::single(error.token_id),
                        format!("{} expected here", Tok::NAME),
                    ))
                    .with_note(Note {
                        kind: NoteKind::Debug,
                        text: format!("at token {token:?}"),
                        suggestion: None,
                    }),
            );
            ParseError::new(TokenSpan::single(error.token_id), self.rule_traceback())
        })
    }

    pub fn expect_token<Tok>(&mut self) -> Result<Tok, ParseError>
    where
        Tok: SingleToken,
    {
        self.expect_token_from(LexicalContext::Default, Channel::CODE)
    }

    /// Returns whether the next token starts `N` without advancing the token stream.
    pub fn next_matches<N>(&mut self) -> bool
    where
        N: PredictiveParse,
    {
        let next_token = self.peek_token_from(
            LexicalContext::Default,
            Channel::CODE | N::LISTEN_TO_CHANNELS,
        );
        #[allow(deprecated)]
        N::started_by(&next_token, &self.sources)
    }

    pub fn scope_mut<R>(&mut self, name: &'static str, f: impl FnOnce(&mut Self) -> R) -> R {
        #[cfg(feature = "parse-traceback")]
        {
            self.rule_traceback.push(name);
            let result = f(self);
            self.rule_traceback.pop();
            result
        }
        #[cfg(not(feature = "parse-traceback"))]
        {
            let _ = name;
            f(self)
        }
    }

    pub fn parse<N>(&mut self) -> Result<N, ParseError>
    where
        N: Parse,
    {
        self.scope_mut(std::any::type_name::<N>(), |parser| {
            #[allow(deprecated)]
            N::parse(parser)
        })
    }

    pub fn parse_with_error<N>(
        &mut self,
        diagnostic: impl FnOnce(&Self, TokenSpan) -> Diagnostic<Token>,
    ) -> Result<N, ParseError>
    where
        N: Parse,
    {
        self.sub(Some(&mut ())).parse().map_err(|error| {
            self.emit_diagnostic(diagnostic(self, error.span));
            error
        })
    }
}

/// The AST node could not be parsed.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: TokenSpan,
    pub rule_traceback: Vec<&'static str>,
}

impl ParseError {
    pub fn new(span: TokenSpan, rule_traceback: Vec<&'static str>) -> Self {
        Self {
            span,
            rule_traceback,
        }
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

impl<T> Parse for Box<T>
where
    T: Parse,
{
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Ok(Box::new(parser.parse()?))
    }
}

pub trait PredictiveParse: Parse {
    /// Additional channels to peek from when parsing this rule predictively.
    const LISTEN_TO_CHANNELS: Channel = Channel::empty();

    /// Returns `true` if this rule starts with the given token.
    #[deprecated = "use [`Parser::next_matches`] instead"]
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool;
}

impl<N> Parse for Option<N>
where
    N: PredictiveParse,
{
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        if parser.next_matches::<N>() {
            Ok(Some(parser.parse()?))
        } else {
            Ok(None)
        }
    }
}
