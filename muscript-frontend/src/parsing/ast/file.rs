use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::{token::EndOfFile, TokenStream},
    parsing::{diagnostics::notes::PARSER_BUG, Parse, ParseError, Parser},
};

use super::{class::Class, Item, TerminatedListErrorKind};

#[derive(Debug, Clone)]
pub enum FileKind {
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct File {
    pub kind: FileKind,
    pub items: Vec<Item>,
    pub eof: EndOfFile,
}

impl Parse for FileKind {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self::Class(parser.parse()?))
    }
}

impl Parse for File {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let kind = parser.parse()?;
        let (items, eof) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "end of file expected after items")
                        .with_label(Label::primary(
                            error.parse.span,
                            "this is where the file should end",
                        ))
                        .with_note(PARSER_BUG),
                ),
            }
            error.parse
        })?;
        Ok(Self { kind, items, eof })
    }
}
