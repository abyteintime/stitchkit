use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    lexis::token::{AnyToken, Greater, Ident, Less, Token},
    list::SeparatedListDiagnostics,
    sources::LexedSources,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{CppBlob, EnumDef, Path, StructDef};

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "type_or_def_error")]
pub enum TypeOrDef {
    StructDef(StructDef),
    EnumDef(EnumDef),
    Type(Type),
}

/// Some variable specifiers are attached to types within the engine source.
#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "specifier_error")]
pub enum TypeSpecifier {
    #[parse(keyword = "const")]
    Const(Ident),
    #[parse(keyword = "native")]
    Native(Ident),
    #[parse(keyword = "transient")]
    Transient(Ident),
}

#[derive(Debug, Clone, Spanned)]
pub struct Type {
    pub specifiers: Vec<TypeSpecifier>,
    pub path: Path,
    pub generic: Option<Generic>,
    pub cpptemplate: Option<CppBlob>,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Generic {
    pub less: Less,
    pub args: Vec<Type>,
    pub greater: Greater,
}

impl Parse for Type {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Ok(Self {
            specifiers: parser.parse_greedy_list()?,
            path: parser.parse()?,
            generic: parser.parse()?,
            cpptemplate: parser.parse()?,
        })
    }
}

impl PredictiveParse for Type {
    #[allow(deprecated)]
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool {
        Ident::started_by(token, sources)
    }
}

impl Parse for Generic {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let less: Less = parser.parse()?;
        let (args, greater) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &less,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `>` to close generics",
                    missing_right_label: "this `<` does not have a matching `>`",
                    missing_comma: "`,` or `>` expected after generic argument",
                    missing_comma_token:
                        "this was expected to continue or close the generic argument list",
                    missing_comma_open: "the generic argument list starts here",
                    missing_comma_note: "note: generic arguments must be separated by commas `,`",
                },
            )
        })?;

        Ok(Self {
            less,
            args,
            greater,
        })
    }
}

impl TypeOrDef {
    pub fn path(&self) -> Path {
        match self {
            TypeOrDef::StructDef(def) => Path::from(def.name),
            TypeOrDef::EnumDef(def) => Path::from(def.name),
            TypeOrDef::Type(ty) => ty.path.clone(),
        }
    }
}

fn specifier_error(parser: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error(format!(
        "unknown type specifier `{}`",
        parser.sources.source(token)
    ))
    .with_label(Label::primary(token, "this specifier is not recognized"))
}

fn type_or_def_error(_: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("type, struct definition, or enum definition expected")
        .with_label(Label::primary(token, "type expected here"))
}
