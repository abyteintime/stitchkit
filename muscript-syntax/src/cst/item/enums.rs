use muscript_foundation::errors::Diagnostic;
use muscript_syntax_derive::Spanned;

use crate::{
    cst::Meta,
    diagnostics::{labels, notes},
    lexis::token::{Ident, LeftBrace, RightBrace, Semi},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KEnum = "enum");

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemEnum {
    pub def: EnumDef,
    pub semi: Option<Semi>,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct EnumDef {
    pub kenum: KEnum,
    pub name: Ident,
    pub open: LeftBrace,
    pub variants: Vec<EnumVariant>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct EnumVariant {
    pub name: Ident,
    pub meta: Option<Meta>,
}

impl Parse for EnumDef {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let kenum = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "enum name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let open = parser.parse()?;
        let (variants, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `}` to close enum variant list",
                    missing_right_label: "this `{` does not have a matching `}`",
                    missing_comma: "`,` or `}` expected after enum variant",
                    missing_comma_open:
                        "this was expected to continue or close the enum variant list",
                    missing_comma_token: "the enum variant list starts here",
                    missing_comma_note: "note: enum variants must be separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            kenum,
            name,
            open,
            variants,
            close,
        })
    }
}
