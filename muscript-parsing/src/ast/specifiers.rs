//! Specifier keywords.

use crate::{
    lexis::token::{LeftParen, RightParen},
    list::DelimitedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::Expr;

keyword! {
    KAbstract = "abstract",
    KCoerce = "coerce",
    KCollapseCategories = "collapsecategories",
    KConfig = "config",
    KConst = "const",
    KDependsOn = "dependson",
    KEditConst = "editconst",
    KEditInlineNew = "editinlinenew",
    KExport = "export",
    KFinal = "final",
    KGlobalConfig = "globalconfig",
    KHideCategories = "hidecategories",
    KImmutable = "immutable",
    KImplements = "implements",
    KInherits = "inherits",
    KInterp = "interp",
    KLocalized = "localized",
    KNative = "native",
    KNativeReplication = "nativereplication",
    KNoClear = "noclear",
    KNoExport = "noexport",
    KOptional = "optional",
    KOut = "out",
    KPrivate = "private",
    KRepNotify = "repnotify",
    KSimulated = "simulated",
    KSkip = "skip",
    KStatic = "static",
    KTransient = "transient",
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct SpecifierArgs {
    pub open: LeftParen,
    pub args: Vec<Expr>,
    pub close: RightParen,
}

impl Parse for SpecifierArgs {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
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
