use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    diagnostics::{labels, notes},
    lexis::{
        token::{Ident, LeftParen, RightParen, Semi, Token},
        TokenStream,
    },
    list::{DelimitedListDiagnostics, TerminatedListErrorKind},
    Parse, ParseError, Parser, PredictiveParse,
};

use super::{KAbstract, KImplements, KInherits, KNative, KNoExport, KTransient};

keyword!(KClass = "class");
keyword!(KExtends = "extends");

#[derive(Debug, Clone, PredictiveParse)]
pub struct Class {
    pub class: KClass,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub specifiers: Vec<ClassSpecifier>,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Extends {
    pub extends: KExtends,
    pub parent_class: Ident,
}

#[derive(Debug, Clone, Parse)]
#[parse(error = "specifier_error")]
pub enum ClassSpecifier {
    Abstract(KAbstract),
    Implements(KImplements, ClassSpecifierArgs),
    Inherits(KInherits, ClassSpecifierArgs),
    Native(KNative, Option<ClassSpecifierArgs>),
    NoExport(KNoExport),
    Transient(KTransient),
}

fn specifier_error(parser: &Parser<'_, impl TokenStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown class specifier `{}`",
            token.span.get_input(parser.input)
        ),
    )
    .with_label(Label::primary(
        token.span,
        "this specifier is not recognized",
    ))
    .with_note("note: notable class specifiers include `placeable` and `abstract`")
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct ClassSpecifierArgs {
    pub open: LeftParen,
    pub args: Vec<Ident>,
    pub close: RightParen,
}

impl Parse for Class {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let class = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "class name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let extends = parser.parse()?;
        let (specifiers, semi) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `;` after class specifier list")
                        .with_label(Label::primary(
                            error.parse.span,
                            "this was expected to be `;`",
                        )),
                ),
            }
            error.parse
        })?;
        Ok(Self {
            class,
            name,
            extends,
            specifiers,
            semi,
        })
    }
}

impl Parse for ClassSpecifierArgs {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftParen = parser.parse()?;
        let (args, close) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(
                &open,
                error,
                DelimitedListDiagnostics {
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
