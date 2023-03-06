use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::Parse;

use crate::{
    lexis::token::{Semi, Token},
    ParseStream, Parser,
};

#[derive(Debug, Clone, Parse)]
#[parse(error = "stmt_error")]
pub enum Stmt {
    Empty(Semi),
}

fn stmt_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "statement expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start a statement",
        ))
        .with_note("note: notable statement types include expressions, `local`, `if`, `while`, `for`, etc.")
}
