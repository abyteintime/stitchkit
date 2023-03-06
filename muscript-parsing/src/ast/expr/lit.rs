use indoc::indoc;
use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{self, Float, Int, IntHex, Name, Token},
    Parse, ParseStream, Parser, PredictiveParse,
};

keyword!(KNone = "none");
keyword!(KTrue = "true");
keyword!(KFalse = "false");

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "bool_lit_error")]
pub enum BoolLit {
    True(KTrue),
    False(KFalse),
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "int_lit_error")]
pub enum IntLit {
    Dec(Int),
    Hex(IntHex),
}

// NOTE: If you want to parse a literal, you actually probably want to use `Expr` instead.
// This lets the user enjoy full expression syntax, as you can const-evaluate the expression
// during semantic analysis. Also, negation `-` is not part of number literals, so beware!
#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "lit_error")]
pub enum Lit {
    None(KNone),
    Bool(BoolLit),
    Int(IntLit),
    Float(Float),
    String(token::String),
    Name(Name),
}

fn bool_lit_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "boolean `true` or `false` expected")
        .with_label(Label::primary(token.span, "this token is not a boolean"))
}

fn int_lit_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "integer literal expected")
        .with_label(Label::primary(
            token.span,
            "this token is not an integer literal",
        ))
        .with_note(
            "note: integer literals can be decimal - like `123` - or hexadecimal - like `0x1ABC`",
        )
}

fn lit_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "literal expected")
        .with_label(Label::primary(token.span, "this token is not a literal"))
        .with_note(indoc!(
            r#"note: literals include
                   - `none`
                   - booleans `true` and `false`
                   - integers `123`, `0xAABBCCDD`
                   - floats   `3.14159265`, `1e-10`
                   - strings  `"Hello, world!"`
                   - names    `'Actor'`
            "#
        ))
        .with_note(indoc!(
            "note: negation `-` is not considered part of literals;
                   therefore if you're reading this, you've probably hit a parser bug.
                   please submit a report at https://github.com/abyteintime/stitchkit"
        ))
}
