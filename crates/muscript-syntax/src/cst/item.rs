mod consts;
mod cpptext;
mod default_properties;
mod enums;
mod function;
mod named;
mod replication;
mod simulated_hack;
mod state;
mod structs;
mod var;

use muscript_foundation::errors::{Diagnostic, Label};
use muscript_lexer::{token::Token, token_stream::TokenStream};
use muscript_syntax_derive::Spanned;

use crate::{
    token::{AnyToken, Semi},
    Parse, Parser,
};

pub use consts::*;
pub use cpptext::*;
pub use default_properties::*;
pub use enums::*;
pub use function::*;
pub use named::*;
pub use replication::*;
pub use simulated_hack::*;
pub use state::*;
pub use structs::*;
pub use var::*;

use super::Stmt;

#[derive(Debug, Clone, Parse, Spanned)]
#[parse(error = "_item_error")]
pub enum Item {
    Empty(Semi),
    Var(ItemVar),
    Const(ItemConst),
    // NOTE: This one needs to be above `function` and `state`.
    Simulated(ItemSimulated),
    Function(ItemFunction),
    Struct(ItemStruct),
    Enum(ItemEnum),
    State(ItemState),
    DefaultProperties(ItemDefaultProperties),
    StructDefaultProperties(ItemStructDefaultProperties),
    Replication(ItemReplication),
    CppText(ItemCppText),
    StructCppText(ItemStructCppText),

    // Same thing as with Stmt's error function - we need a way of overriding the error message
    // here.
    #[parse(fallback)]
    Stmt(Stmt),
}

fn _item_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("item expected")
        .with_label(Label::primary(token, "this token does not start an item"))
        .with_note("help: notable types of items include `var`, `function`, `struct`, and `enum`")
}
