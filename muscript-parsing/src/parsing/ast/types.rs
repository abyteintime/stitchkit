use muscript_parsing_derive::PredictiveParse;

use crate::{
    lexis::{
        token::{Greater, Ident, Less},
        TokenStream,
    },
    parsing::{list::DelimitedListDiagnostics, Parse, ParseError, Parser},
};

#[derive(Debug, Clone, PredictiveParse)]
pub struct Type {
    pub name: Ident,
    pub generic: Option<Generic>,
}

#[derive(Debug, Clone, PredictiveParse)]
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
            parser.emit_delimited_list_diagnostic(
                error,
                DelimitedListDiagnostics {
                    missing_left: "generics `<T, U, ..>` expected",
                    missing_left_label: "generic arguments expected here",
                    missing_right: "missing `>` to close generics",
                    missing_comma: "`,` or `>` expected after generic argument",
                    missing_comma_token:
                        "this was expected to continue or close the generic argument list",
                    missing_comma_open: "the generic argument list starts here",
                    missing_comma_note: "note: generic arguments must be separated by commas `,`",
                },
            )
        })?;

        Ok(Self {
            less,
            args,
            greater,
        })
    }
}
