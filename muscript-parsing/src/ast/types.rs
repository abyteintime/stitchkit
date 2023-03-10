use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{Greater, Ident, Less, Token},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{EnumDef, StructDef};

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "type_or_def_error")]
pub enum TypeOrDef {
    StructDef(StructDef),
    EnumDef(EnumDef),
    Type(Type),
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Type {
    pub name: Ident,
    pub generic: Option<Generic>,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Generic {
    pub less: Less,
    pub args: Vec<Type>,
    pub greater: Greater,
}

impl Parse for Type {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Ok(Self {
            name: parser.parse()?,
            generic: parser.parse()?,
        })
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

fn type_or_def_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        "type, struct definition, or enum definition expected",
    )
    .with_label(Label::primary(token.span, "type expected here"))
}
