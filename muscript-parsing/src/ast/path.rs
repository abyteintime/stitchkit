use crate::{
    lexis::token::{Ident, Token, TokenKind},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

#[derive(Debug, Clone)]
pub struct Path {
    pub components: Vec<Ident>,
}

impl Path {
    pub fn continue_parsing(
        parser: &mut Parser<'_, impl ParseStream>,
        root: Ident,
    ) -> Result<Self, ParseError> {
        let mut components = vec![root];
        while parser.peek_token()?.kind == TokenKind::Dot {
            let _dot = parser.next_token()?;
            components.push(parser.parse()?);
        }
        Ok(Self { components })
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
