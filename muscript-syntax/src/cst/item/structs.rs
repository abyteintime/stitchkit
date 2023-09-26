use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    cst::{CppBlob, Extends},
    diagnostics::{labels, notes},
    lexis::token::{AnyToken, Ident, LeftBrace, RightBrace, Semi, Token},
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::Item;

keyword!(KStruct = "struct");

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemStruct {
    pub def: StructDef,
    // UX thing: MuScript considers the semicolon after `}` optional.
    pub semi: Option<Semi>,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct StructDef {
    pub kstruct: KStruct,
    pub specifiers: Vec<StructSpecifier>,
    pub cpp_name: Option<CppBlob>,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub open: LeftBrace,
    pub items: Vec<Item>,
    pub close: RightBrace,
}

impl Parse for StructDef {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let kstruct = parser.parse()?;
        let specifiers = parser.parse_greedy_list()?;
        let cpp_name = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error("struct name expected")
                .with_label(labels::invalid_identifier(span, &parser.sources))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let extends = parser.parse()?;
        let open: LeftBrace = parser.parse()?;
        let (items, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error("missing `}` to close struct body").with_label(
                        Label::primary(&open, "this is where the struct body begins"),
                    ),
                )
            }
            error.parse
        })?;
        Ok(Self {
            kstruct,
            specifiers,
            name,
            extends,
            cpp_name,
            open,
            items,
            close,
        })
    }
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "specifier_error")]
pub enum StructSpecifier {
    #[parse(keyword = "export")]
    Export(Ident),
    #[parse(keyword = "immutable")]
    Immutable(Ident),
    #[parse(keyword = "immutablewhencooked")]
    ImmutableWhenCooked(Ident),
    #[parse(keyword = "native")]
    Native(Ident),
    #[parse(keyword = "transient")]
    Transient(Ident),
}

fn specifier_error(parser: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error(format!(
        "unknown struct specifier `{}`",
        parser.sources.source(token)
    ))
    .with_label(Label::primary(token, "this specifier is not recognized"))
    .with_note("note: notable struct specifiers include `immutable` and `transient`")
}
