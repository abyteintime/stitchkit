mod consts;
mod cpptext;
mod default_properties;
mod enums;
mod function;
mod replication;
mod simulated_hack;
mod state;
mod structs;
mod var;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{Semi, Token},
    Parse, ParseStream, Parser,
};

pub use consts::*;
pub use cpptext::*;
pub use default_properties::*;
pub use enums::*;
pub use function::*;
pub use replication::*;
pub use simulated_hack::*;
pub use state::*;
pub use structs::*;
pub use var::*;

use super::Stmt;

#[derive(Debug, Clone, Parse)]
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

fn _item_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "item expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start an item",
        ))
        .with_note("help: notable types of items include `var`, `function`, `struct`, and `enum`")
}
