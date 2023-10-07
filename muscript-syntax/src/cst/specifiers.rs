//! Specifier keywords.

use muscript_lexer::token_stream::TokenStream;
use muscript_syntax_derive::Spanned;

use crate::{
    list::SeparatedListDiagnostics,
    token::{LeftParen, RightParen},
    Parse, ParseError, Parser, PredictiveParse,
};

use super::Expr;

keyword! {
    KConst = "const",
    KSimulated = "simulated",
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct SpecifierArgs {
    pub open: LeftParen,
    pub args: Vec<Expr>,
    pub close: RightParen,
}

impl Parse for SpecifierArgs {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftParen = parser.parse()?;
        let (args, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `)` to close specifier argument list",
                    missing_right_label: "this `(` does not have a matching `)`",
                    missing_comma: "`,` or `)` expected after specifier argument",
                    missing_comma_open: "the specifier argument list starts here",
                    missing_comma_token:
                        "this was expected to continue or close the specifier argument list",
                    missing_comma_note: "note: specifier arguments must be separated by commas `,`",
                },
            )
        })?;
        Ok(Self { open, args, close })
    }
}
