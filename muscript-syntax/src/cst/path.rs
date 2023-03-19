use std::borrow::Cow;

use muscript_syntax_derive::Spanned;

use crate::{
    lexis::token::{Ident, Token, TokenKind},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
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
        parser: &mut Parser<'_, impl ParseStream>,
        root: Ident,
    ) -> Result<Self, ParseError> {
        let mut components = vec![root];
        while parser.peek_token()?.kind == TokenKind::Dot {
            let _dot = parser.next_token()?;
            components.push(parser.parse()?);
        }
        Ok(Self::new(components))
    }

    pub fn pretty_print<'a>(&self, input: &'a str) -> Cow<'a, str> {
        if let [first] = &self.components[..] {
            Cow::Borrowed(first.span.get_input(input))
        } else {
            let mut buffer = String::new();
            for (i, component) in self.components.iter().enumerate() {
                if i != 0 {
                    buffer.push('.');
                }
                buffer.push_str(component.span.get_input(input));
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
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let root = parser.parse()?;
        Self::continue_parsing(parser, root)
    }
}

impl PredictiveParse for Path {
    fn started_by(token: &Token, _: &str) -> bool {
        token.kind == TokenKind::Ident
    }
}
