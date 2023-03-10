mod consts;
mod cpptext;
mod default_properties;
mod enums;
mod function;
mod replication;
mod structs;
mod var;

use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::Parse;

use crate::{
    lexis::token::{Semi, Token},
    ParseStream, Parser,
};

pub use consts::*;
pub use cpptext::*;
pub use default_properties::*;
pub use enums::*;
pub use function::*;
pub use replication::*;
pub use structs::*;
pub use var::*;

#[derive(Debug, Clone, Parse)]
#[parse(error = "item_error")]
pub enum Item {
    Empty(Semi),
    Var(ItemVar),
    Const(ItemConst),
    Function(ItemFunction),
    Struct(ItemStruct),
    Enum(ItemEnum),
    DefaultProperties(ItemDefaultProperties),
    StructDefaultProperties(ItemStructDefaultProperties),
    Replication(ItemReplication),
    CppText(ItemCppText),
    StructCppText(ItemStructCppText),
}

fn item_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "item expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start an item",
        ))
        .with_note("help: notable types of items include `var`, `function`, `struct`, and `enum`")
}
