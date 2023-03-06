use indoc::indoc;
use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{
        self, Assign, Float, Ident, LeftBrace, LeftParen, RightBrace, RightParen, Semi, Token,
    },
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::IntLit;

#[derive(Debug, Clone, PredictiveParse)]
pub struct DefaultPropertiesBlock {
    pub open: LeftBrace,
    pub properties: Vec<DefaultProperty>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "default_property_error")]
pub enum DefaultProperty {
    KeyValuePair(KeyValuePair),
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct KeyValuePair {
    pub key: Key,
    pub equals: Assign,
    pub value: Lit,
    pub semi: Option<Semi>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Key {
    pub ident: Ident,
    pub index: Option<Index>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Index {
    pub open: LeftParen,
    pub index: IntLit,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "lit_error")]
pub enum Lit {
    Int(IntLit),
    Float(Float),
    String(token::String),
    Ident(Ident),
}

impl Parse for DefaultPropertiesBlock {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse()?;
        let (properties, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `}` to close default properties block")
                        .with_label(Label::primary(open.span, "this is where the block begins")),
                );
            }
            error.parse
        })?;
        Ok(Self {
            open,
            properties,
            close,
        })
    }
}

fn default_property_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "default property expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start a default property",
        ))
        .with_note(indoc!(
            r#"note: default property declarations can take one of the following forms:
                   - `Key = Value`
                   - `begin object .. end object` (case-insensitive)
                   - `Array.Operation(OptionalArgs)`
            "#,
        ))
}

fn lit_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "default property literal expected")
        .with_label(Label::primary(
            token.span,
            "this token is not a supported literal",
        ))
        .with_note("note: values of default properties must be literal and cannot be expressions")
        .with_note(indoc!(
            r#"note: the syntax of certain literals is different from normal UnrealScript
                   - name literals are not enclosed in apostrophes;
                     for `var name Example;` its default value can be specified using `Example = Something`
                   - unlike in normal code, arrays and structs have literals
                     arrays:  `(1, 2, 3)`
                     structs: `(X=1, Y=2, Z=3)` - instead of `vect(1, 2, 3)`
            "#
        ))
}
