mod function;
mod structs;
mod var;

use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::Parse;

use crate::{
    lexis::{token::Token, TokenStream},
    Parser,
};

pub use function::*;
pub use structs::*;
pub use var::*;

#[derive(Debug, Clone, Parse)]
#[parse(error = "item_error")]
pub enum Item {
    Var(ItemVar),
    Function(ItemFunction),
    Struct(ItemStruct),
}

fn item_error(parser: &Parser<'_, impl TokenStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "item expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start an item",
        ))
        .with_note("help: notable types of items include `var`, `function`, `struct`, and `enum`")
}
