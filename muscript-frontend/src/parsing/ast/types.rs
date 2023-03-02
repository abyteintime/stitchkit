use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{
        token::{Greater, Ident, Less, Token, TokenKind},
        TokenStream,
    },
    parsing::{Parse, ParseError, Parser, PredictiveParse},
};

use super::DelimitedListErrorKind;

#[derive(Debug, Clone)]
pub struct Type {
    pub name: Ident,
    pub generic: Option<Generic>,
}

#[derive(Debug, Clone)]
pub struct Generic {
    pub less: Less,
    pub args: Vec<Type>,
    pub greater: Greater,
}

impl Parse for Type {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            name: parser.parse()?,
            generic: parser.parse()?,
        })
    }
}

impl Parse for Generic {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let (less, args, greater) = parser.parse_delimited_list().map_err(|error| {
            match error.kind {
                DelimitedListErrorKind::Parse => (),
                DelimitedListErrorKind::MissingLeft => {
                    parser.emit_diagnostic(
                        Diagnostic::error(parser.file, "generics `<T, U, ..>` expected")
                            .with_label(Label::primary(
                                error.parse.span,
                                "generic arguments expected here",
                            )),
                    );
                }
                DelimitedListErrorKind::MissingRight { open } => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `>` to close generics")
                        .with_label(Label::secondary(
                            open,
                            "the generic argument list starts here...",
                        ))
                        .with_label(Label::primary(
                            error.parse.span,
                            "...and was expected to end here",
                        )),
                ),
                DelimitedListErrorKind::MissingComma { open } => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "`,` or `>` expected after generic argument")
                        .with_label(Label::primary(
                            error.parse.span,
                            "this was expected to continue or close the generic argument list",
                        ))
                        .with_label(Label::secondary(
                            open,
                            "the generic argument list starts here",
                        ))
                        .with_note("note: generic arguments must be separated by commas `,`"),
                ),
            }
            error.parse
        })?;

        Ok(Self {
            less,
            args,
            greater,
        })
    }
}

impl PredictiveParse for Generic {
    fn starts_with(token: &Token, _: &str) -> bool {
        token.kind == TokenKind::Less
    }
}
