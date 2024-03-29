use indoc::indoc;
use muscript_foundation::errors::{Diagnostic, Label};
use muscript_lexer::{token::Token, token_stream::TokenStream};
use muscript_syntax_derive::Spanned;

use crate::{
    token::{AnyToken, FloatLit, IntLit, NameLit, StringLit},
    Parse, Parser, PredictiveParse,
};

keyword!(KNone = "none");
keyword!(KTrue = "true");
keyword!(KFalse = "false");

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "bool_lit_error")]
pub enum BoolLit {
    True(KTrue),
    False(KFalse),
}

// NOTE: If you want to parse a literal, you actually probably want to use `Expr` instead.
// This lets the user enjoy full expression syntax, as you can const-evaluate the expression
// during semantic analysis. Also, negation `-` is not part of number literals, so beware!
#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "lit_error")]
pub enum Lit {
    None(KNone),
    Bool(BoolLit),
    Int(IntLit),
    Float(FloatLit),
    String(StringLit),
    Name(NameLit),
}

fn bool_lit_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("boolean `true` or `false` expected")
        .with_label(Label::primary(token, "this token is not a boolean"))
}

fn lit_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("literal expected")
        .with_label(Label::primary(token, "this token is not a literal"))
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
