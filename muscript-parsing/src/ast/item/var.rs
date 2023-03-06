use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    ast::{Expr, KConst, KEditConst, KNative, KNoExport, KPrivate, KTransient, Type},
    lexis::token::{Ident, LeftBracket, LeftParen, RightBracket, RightParen, Semi, Token},
    list::DelimitedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KVar = "var");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemVar {
    pub var: KVar,
    pub editor: Option<VarEditor>,
    pub specifiers: Vec<VarSpecifier>,
    pub ty: Type,
    pub variables: Vec<VarDef>,
    pub semi: Semi,
}

impl Parse for ItemVar {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let var = parser.parse()?;
        let editor = parser.parse()?;
        let specifiers = parser.parse_greedy_list()?;
        let ty = parser.parse()?;
        let (names, semi) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(
                &var,
                error,
                DelimitedListDiagnostics {
                    missing_right: "missing `;` to end variable declaration",
                    missing_right_label: "this variable declaration does not have a `;`",
                    missing_comma: "`,` or `;` expected after variable name",
                    missing_comma_open:
                        "this is the variable declaration",
                    missing_comma_token: "this was expected to continue or end the variable declaration",
                    missing_comma_note:
                        "note: multiple variable names in one `var` must be separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            var,
            editor,
            specifiers,
            ty,
            variables: names,
            semi,
        })
    }
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarEditor {
    pub open: LeftParen,
    pub category: Option<Ident>,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "specifier_error")]
pub enum VarSpecifier {
    Const(KConst),
    EditConst(KEditConst),
    Native(KNative),
    NoExport(KNoExport),
    Private(KPrivate),
    Transient(KTransient),
}

fn specifier_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown variable specifier `{}`",
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

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarDef {
    pub name: Ident,
    pub array: Option<VarArray>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarArray {
    pub open: LeftBracket,
    pub size: Expr,
    pub close: RightBracket,
}
