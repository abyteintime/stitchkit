use crate::{
    lexis::{token::EndOfFile, TokenStream},
    parsing::{Parse, ParseError, Parser},
};

use super::class::Class;

#[derive(Debug, Clone)]
pub enum FileKind {
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct File {
    pub kind: FileKind,
    pub eof: EndOfFile,
}

impl Parse for FileKind {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self::Class(parser.parse()?))
    }
}

impl Parse for File {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            kind: parser.parse()?,
            // In normal circumstances "missing end of file" errors shouldn't really happen,
            eof: parser.parse()?,
        })
    }
}
