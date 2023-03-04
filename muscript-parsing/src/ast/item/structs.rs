use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::Spanned,
};
use muscript_parsing_derive::PredictiveParse;

use crate::{
    diagnostics::{labels, notes},
    lexis::{
        token::{Ident, LeftBrace, RightBrace, Semi},
        TokenStream,
    },
    list::TerminatedListErrorKind,
    Parse, ParseError, Parser,
};

use super::Item;

keyword!(KStruct = "struct");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemStruct {
    pub kstruct: KStruct,
    pub name: Ident,
    pub open: LeftBrace,
    pub items: Vec<Item>,
    pub close: RightBrace,
    // UX thing: MuScript considers the semicolon after `}` optional.
    pub semi: Option<Semi>,
}

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let kstruct = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "struct name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let open: LeftBrace = parser.parse()?;
        let (items, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `}` to close struct body").with_label(
                        Label::primary(open.span(), "this is where the struct body begins"),
                    ),
                )
            }
            error.parse
        })?;
        let semi = parser.parse()?;
        Ok(Self {
            kstruct,
            name,
            open,
            items,
            close,
            semi,
        })
    }
}
