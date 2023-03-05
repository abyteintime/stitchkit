use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::Spanned,
};
use muscript_parsing_derive::PredictiveParse;

use crate::{
    ast::{CppBlob, Extends, KImmutable},
    diagnostics::{labels, notes},
    lexis::{
        token::{Ident, LeftBrace, RightBrace, Semi, Token},
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
    pub specifiers: Vec<StructSpecifier>,
    pub cpp_name: Option<CppBlob>,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub open: LeftBrace,
    pub items: Vec<Item>,
    pub close: RightBrace,
    // UX thing: MuScript considers the semicolon after `}` optional.
    pub semi: Option<Semi>,
}

impl Parse for ItemStruct {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let kstruct = parser.parse()?;
        let specifiers = parser.parse_greedy_list()?;
        let cpp_name = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "struct name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let extends = parser.parse()?;
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
            specifiers,
            name,
            extends,
            cpp_name,
            open,
            items,
            close,
            semi,
        })
    }
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "specifier_error")]
pub enum StructSpecifier {
    Immutable(KImmutable),
}

fn specifier_error(parser: &Parser<'_, impl TokenStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown struct specifier `{}`",
            token.span.get_input(parser.input)
        ),
    )
    .with_label(Label::primary(
        token.span,
        "this specifier is not recognized",
    ))
    // TODO: After we have most specifiers, list notable ones here.
    // .with_note("note: notable variable specifiers include [what?]")
}
