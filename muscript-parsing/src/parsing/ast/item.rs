mod function;
mod var;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::TokenStream,
    parsing::{Parse, ParseError, Parser, PredictiveParse},
};

pub use function::*;
pub use var::*;

#[derive(Debug, Clone)]
pub enum Item {
    Var(ItemVar),
    Function(ItemFunction),
}

impl Parse for Item {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let token = parser.peek_token()?;
        Ok(match token {
            _ if ItemVar::started_by(&token, parser.input) => Item::Var(parser.parse()?),
            _ if ItemFunction::started_by(&token, parser.input) => Item::Function(parser.parse()?),
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
