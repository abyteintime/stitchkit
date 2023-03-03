use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{
        token::{Ident, Semi, Token},
        TokenStream,
    },
    parsing::{Parse, ParseError, Parser, PredictiveParse},
};

use super::Type;

keyword!(KVar = "var");

#[derive(Debug, Clone)]
pub struct ItemVar {
    pub var: KVar,
    pub ty: Type,
    pub name: Ident,
    pub semi: Semi,
}

impl Parse for ItemVar {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            // TODO: Error recovery here; if any of those are incorrect, we should read until the
            // next semicolon.
            var: parser.parse()?,
            ty: parser.parse()?,
            name: parser.parse()?,
            semi: parser.parse()?,
        })
    }
}

impl PredictiveParse for ItemVar {
    fn starts_with(token: &Token, input: &str) -> bool {
        KVar::starts_with(token, input)
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Var(ItemVar),
}

impl Parse for Item {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let token = parser.peek_token()?;
        Ok(match token {
            _ if ItemVar::starts_with(&token, parser.input) => Item::Var(parser.parse()?),
            _ => parser.bail(
                token.span,
                Diagnostic::error(parser.file, "item expected").with_label(Label::primary(
                    token.span,
                    "this token does not start an item",
                )).with_note("help: notable types of items include `var`, `function`, `struct`, and `enum`"),
            )?,
        })
    }
}
