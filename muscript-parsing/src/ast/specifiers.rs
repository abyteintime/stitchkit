//! Specifier keywords.

use crate::{
    lexis::token::{LeftParen, RightParen},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::Expr;

keyword! {
    KAbstract = "abstract",
    KAutoExpandCategories = "autoexpandcategories",
    KBitWise = "bitwise",
    KClassGroup = "classgroup",
    KCoerce = "coerce",
    KCollapseCategories = "collapsecategories",
    KConfig = "config",
    KConst = "const",
    KDependsOn = "dependson",
    KDeprecated = "deprecated",
    KEditConst = "editconst",
    KEditInlineNew = "editinlinenew",
    KEditorOnly = "editoronly",
    KExec = "exec",
    KExport = "export",
    KFinal = "final",
    KGlobalConfig = "globalconfig",
    KHideCategories = "hidecategories",
    KImmutable = "immutable",
    KImmutableWhenCooked = "immutablewhencooked",
    KImplements = "implements",
    KInherits = "inherits",
    KInterp = "interp",
    KIterator = "iterator",
    KLatent = "latent",
    KLocalized = "localized",
    KNative = "native",
    KNativeReplication = "nativereplication",
    KNoClear = "noclear",
    KNoExport = "noexport",
    KNoImport = "noimport",
    KOptional = "optional",
    KOut = "out",
    KPrivate = "private",
    KProtected = "protected",
    KPlaceable = "placeable",
    KPublic = "public",
    KReliable = "reliable",
    KRepNotify = "repnotify",
    KServer = "server",
    KSimulated = "simulated",
    KSkip = "skip",
    KStatic = "static",
    KTransient = "transient",
    KVirtual = "virtual",
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
