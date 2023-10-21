use std::borrow::Cow;

use muscript_lexer::{sources::LexedSources, token::TokenKind, token_stream::TokenStream};
use muscript_syntax_derive::Spanned;

use crate::{
    token::{AnyToken, Ident},
    Parse, ParseError, Parser, PredictiveParse,
};

#[derive(Debug, Clone, Spanned)]
pub struct Path {
    pub components: Vec<Ident>,
}

impl Path {
    pub fn new(components: Vec<Ident>) -> Self {
        Self { components }
    }

    pub fn continue_parsing(
        parser: &mut Parser<'_, impl TokenStream>,
        root: Ident,
    ) -> Result<Self, ParseError> {
        let mut components = vec![root];
        while parser.peek_token().kind == TokenKind::Dot {
            let _dot = parser.next_token();
            components.push(parser.parse()?);
        }
        Ok(Self::new(components))
    }

    pub fn pretty_print<'a>(&self, sources: &LexedSources<'a>) -> Cow<'a, str> {
        if let [first] = &self.components[..] {
            Cow::Borrowed(sources.source(first))
        } else {
            let mut buffer = String::new();
            for (i, component) in self.components.iter().enumerate() {
                if i != 0 {
                    buffer.push('.');
                }
                buffer.push_str(sources.source(component));
            }
            Cow::Owned(buffer)
        }
    }
}

impl From<Ident> for Path {
    fn from(value: Ident) -> Self {
        Self::new(vec![value])
    }
}

impl Parse for Path {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let root = parser.parse()?;
        Self::continue_parsing(parser, root)
    }
}

impl PredictiveParse for Path {
    fn started_by(token: &AnyToken, _: &LexedSources<'_>) -> bool {
        token.kind == TokenKind::Ident
    }
}
