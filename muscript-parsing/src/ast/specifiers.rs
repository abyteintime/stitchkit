//! Specifier keywords.

use crate::{
    lexis::{
        token::{LeftParen, RightParen},
        TokenStream,
    },
    list::DelimitedListDiagnostics,
    Parse, ParseError, Parser, PredictiveParse,
};

use super::Expr;

keyword!(KAbstract = "abstract");
keyword!(KConst = "const");
keyword!(KEditConst = "editconst");
keyword!(KFinal = "final");
keyword!(KImmutable = "immutable");
keyword!(KImplements = "implements");
keyword!(KInherits = "inherits");
keyword!(KNative = "native");
keyword!(KNoExport = "noexport");
keyword!(KPrivate = "private");
keyword!(KStatic = "static");
keyword!(KTransient = "transient");

#[derive(Debug, Clone, PredictiveParse)]
pub struct SpecifierArgs {
    pub open: LeftParen,
    pub args: Vec<Expr>,
    pub close: RightParen,
}

impl Parse for SpecifierArgs {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftParen = parser.parse()?;
        let (args, close) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(
                &open,
                error,
                DelimitedListDiagnostics {
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
