use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{
        token::{Int, IntHex, Token},
        TokenStream,
    },
    Parse, Parser, PredictiveParse,
};

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "int_lit_error")]
pub enum IntLit {
    Dec(Int),
    Hex(IntHex),
}

fn int_lit_error(parser: &Parser<'_, impl TokenStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "integer literal expected")
        .with_label(Label::primary(
            token.span,
            "this token is not an integer literal",
        ))
        .with_note(
            "note: integer literals can be decimal - like `123` - or hexadecimal - like `0x1ABC`",
        )
}
