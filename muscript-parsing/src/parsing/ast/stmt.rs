use crate::{
    lexis::{token::Semi, TokenStream},
    parsing::{Parse, ParseError, Parser},
};

#[derive(Debug, Clone)]
pub enum Stmt {
    Empty(Semi),
}

impl Parse for Stmt {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self::Empty(parser.parse()?))
    }
}
