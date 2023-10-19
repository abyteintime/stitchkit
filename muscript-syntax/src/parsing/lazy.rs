use muscript_foundation::{
    errors::{Diagnostic, Label},
    span::Spanned,
};
use muscript_lexer::{
    sources::LexedSources,
    token::{AnyToken, Token, TokenKind, TokenSpan},
    token_stream::TokenStream,
};

use crate::{
    token::{LeftBrace, RightBrace, SingleToken},
    Parse, ParseError, Parser, PredictiveParse,
};

pub trait Delimiters {
    type Open: SingleToken;
    type Close: SingleToken;

    fn new(open: Self::Open, close: Self::Close) -> Self;

    fn open(&self) -> Self::Open;
    fn close(&self) -> Self::Close;
}

#[derive(Debug, Clone, Copy)]
pub struct Braces {
    pub open: LeftBrace,
    pub close: RightBrace,
}

impl Delimiters for Braces {
    type Open = LeftBrace;
    type Close = RightBrace;

    fn new(open: Self::Open, close: Self::Close) -> Self {
        Self { open, close }
    }

    fn open(&self) -> Self::Open {
        self.open
    }

    fn close(&self) -> Self::Close {
        self.close
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LazyBlock<D> {
    pub delimiters: D,
    pub inner: TokenSpan,
}

impl<D> Parse for LazyBlock<D>
where
    D: Delimiters,
{
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: D::Open = parser.parse()?;
        let open_nesting_level = parser.nesting_level();

        let mut inner = TokenSpan::Empty;
        let mut close = None;

        while parser.nesting_level() >= open_nesting_level || open_nesting_level == 0 {
            let token = parser.next_token();
            if let Ok(c) = D::Close::try_from_token(token, &parser.sources) {
                close = Some(c);
            } else if token.kind == TokenKind::EndOfFile {
                parser.emit_diagnostic(missing_closing_delimiter::<D>(&open));
                return Err(parser.make_error(TokenSpan::single(open.id()).join(&inner)));
            }
            inner = inner.join(&TokenSpan::single(token.id));
        }

        if let Some(close) = close {
            Ok(Self {
                delimiters: D::new(open, close),
                inner,
            })
        } else {
            parser.emit_diagnostic(missing_closing_delimiter::<D>(&open));
            Err(parser.make_error(TokenSpan::single(open.id()).join(&inner)))
        }
    }
}

fn missing_closing_delimiter<D>(open: &D::Open) -> Diagnostic<Token>
where
    D: Delimiters,
{
    Diagnostic::error(format!(
        "missing {} to close {}",
        D::Close::NAME,
        D::Open::NAME
    ))
    .with_label(Label::primary(
        open,
        format!("this {} is missing its closing delimiter", D::Open::NAME),
    ))
}

impl<D> PredictiveParse for LazyBlock<D>
where
    D: Delimiters,
{
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool {
        #[allow(deprecated)]
        D::Open::started_by(token, sources)
    }
}

impl<D> Spanned<Token> for LazyBlock<D>
where
    D: Delimiters,
{
    fn span(&self) -> TokenSpan {
        self.delimiters
            .open()
            .span()
            .join(&self.inner)
            .join(&self.delimiters.close().span())
    }
}
