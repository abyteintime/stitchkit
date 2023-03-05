mod lit;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{
        token::{Float, Token},
        TokenStream,
    },
    Parse, Parser, PredictiveParse,
};

pub use lit::*;

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "expr_error")]
pub enum Expr {
    IntLit(IntLit),
    FloatLit(Float),
}

fn expr_error(parser: &Parser<'_, impl TokenStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "expression expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start a expression",
        ))
        .with_note("note: notable expression types include literals, variables, math, etc.")
}
