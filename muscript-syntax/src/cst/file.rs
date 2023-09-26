use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    diagnostics::notes, lexis::token::EndOfFile, list::TerminatedListErrorKind, Parse, ParseError,
    ParseStream, Parser,
};

use super::{Class, Item};

#[derive(Debug, Clone, Spanned, Parse)]
pub struct File {
    pub class: Class,
    pub bare: BareFile,
}

#[derive(Debug, Clone, Spanned)]
pub struct BareFile {
    pub items: Vec<Item>,
    pub eof: EndOfFile,
}

impl Parse for BareFile {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let (items, eof) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error("end of file expected after items")
                        .with_label(Label::primary(
                            &error.parse.span,
                            "this is where the file should end",
                        ))
                        .with_note(notes::PARSER_BUG),
                ),
            }
            error.parse
        })?;
        Ok(Self { items, eof })
    }
}
