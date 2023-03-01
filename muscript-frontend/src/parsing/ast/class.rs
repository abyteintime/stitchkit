use muscript_foundation::errors::Diagnostic;

use crate::{
    lexis::{
        token::{Ident, Semi, Token},
        TokenStream,
    },
    parsing::diagnostics::{labels, notes},
};

use crate::parsing::{Parse, ParseError, Parser, PredictiveParse};

keyword!(KClass = "class");
keyword!(KExtends = "extends");

#[derive(Debug, Clone)]
pub struct Class {
    pub class: KClass,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub semi: Semi,
}

#[derive(Debug, Clone)]
pub struct Extends {
    pub extends: KExtends,
    pub parent_class: Ident,
}

impl Parse for Class {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            class: parser.parse()?,
            name: parser.parse_with_error(|parser, span| {
                Diagnostic::error(parser.file, "class name expected")
                    .with_label(labels::invalid_identifier(span, parser.input))
                    .with_note(notes::IDENTIFIER_CHARS)
            })?,
            extends: parser.parse()?,
            // TODO: Specifiers.
            semi: parser.parse()?,
        })
    }
}

impl Parse for Extends {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            extends: parser.parse()?,
            parent_class: parser.parse()?,
        })
    }
}

impl PredictiveParse for Extends {
    fn starts_with(token: &Token, input: &str) -> bool {
        KExtends::starts_with(token, input)
    }
}
