use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::{Parse, PredictiveParse};

use crate::{
    lexis::{
        token::{Ident, LeftParen, RightParen, Semi},
        TokenStream,
    },
    parsing::{Parse, ParseError, Parser, PredictiveParse},
};

use super::Type;

keyword!(KVar = "var");

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemVar {
    pub var: KVar,
    pub editor: Option<VarEditor>,
    pub ty: Type,
    pub name: Ident,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarEditor {
    pub open: LeftParen,
    pub category: Option<Ident>,
    pub close: RightParen,
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
