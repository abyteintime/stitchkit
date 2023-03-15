use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    cst::{CppBlob, Expr, Meta, TypeOrDef, TypeSpecifier},
    diagnostics,
    lexis::token::{Ident, LeftBracket, LeftParen, RightBracket, RightParen, Semi, Token},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KVar = "var");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemVar {
    pub var: KVar,
    pub editor: Option<VarEditor>,
    pub specifiers: Vec<VarSpecifier>,
    pub ty: TypeOrDef,
    pub variables: Vec<VarDef>,
    pub semi: Semi,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct VarEditor {
    pub open: LeftParen,
    pub categories: Vec<Ident>,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "specifier_error")]
pub enum VarSpecifier {
    #[parse(keyword = "bitwise")]
    BitWise(Ident),
    #[parse(keyword = "config")]
    Config(Ident),
    #[parse(keyword = "crosslevelactive")]
    CrossLevelActive(Ident),
    #[parse(keyword = "crosslevelpassive")]
    CrossLevelPassive(Ident),
    #[parse(keyword = "databinding")]
    DataBinding(Ident),
    #[parse(keyword = "deprecated")]
    Deprecated(Ident),
    #[parse(keyword = "duplicatetransient")]
    DuplicateTransient(Ident),
    #[parse(keyword = "editconst")]
    EditConst(Ident),
    #[parse(keyword = "edithide")]
    EditHide(Ident),
    #[parse(keyword = "editfixedsize")]
    EditFixedSize(Ident),
    #[parse(keyword = "editinline")]
    EditInline(Ident),
    #[parse(keyword = "editinlineuse")]
    EditInlineUse(Ident),
    #[parse(keyword = "editoronly")]
    EditorOnly(Ident),
    #[parse(keyword = "edittextbox")]
    EditTextBox(Ident),
    #[parse(keyword = "export")]
    Export(Ident),
    #[parse(keyword = "globalconfig")]
    GlobalConfig(Ident),
    #[parse(keyword = "init")]
    Init(Ident),
    #[parse(keyword = "input")]
    Input(Ident),
    #[parse(keyword = "instanced")]
    Instanced(Ident),
    #[parse(keyword = "interp")]
    Interp(Ident),
    #[parse(keyword = "localized")]
    Localized(Ident),
    #[parse(keyword = "noclear")]
    NoClear(Ident),
    #[parse(keyword = "noexport")]
    NoExport(Ident),
    #[parse(keyword = "noimport")]
    NoImport(Ident),
    #[parse(keyword = "nontransactional")]
    NonTransactional(Ident),
    #[parse(keyword = "notforconsole")]
    NotForConsole(Ident),
    #[parse(keyword = "private")]
    Private(Ident, Option<CppBlob>),
    #[parse(keyword = "privatewrite")]
    PrivateWrite(Ident),
    #[parse(keyword = "protected")]
    Protected(Ident, Option<CppBlob>),
    #[parse(keyword = "protectedwrite")]
    ProtectedWrite(Ident, Option<CppBlob>),
    #[parse(keyword = "public")]
    Public(Ident, Option<CppBlob>),
    #[parse(keyword = "repnotify")]
    RepNotify(Ident),
    #[parse(keyword = "serialize")]
    Serailize(Ident),
    #[parse(keyword = "serializetext")]
    SerializeText(Ident),
    Type(TypeSpecifier),
}

impl Parse for ItemVar {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let var = parser.parse()?;
        let editor = parser.parse()?;
        let specifiers = parser.parse_greedy_list()?;
        let ty = parser.parse()?;
        let (names, semi) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(&var, error, diagnostics::sets::VARIABLES)
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

impl Parse for VarEditor {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open = parser.parse()?;
        let (categories, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `)` to close category list",
                    missing_right_label: "this `(` does not have a matching `)`",
                    missing_comma: "`,` or `)` expected after category name",
                    missing_comma_open: "this is where the category list begins",
                    missing_comma_token: "this was expected to continue or close the category list",
                    missing_comma_note: "note: category names must not contain spaces",
                },
            )
        })?;
        Ok(Self {
            open,
            categories,
            close,
        })
    }
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
    pub meta: Option<Meta>,
    pub cpptype: Option<CppBlob>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarArray {
    pub open: LeftBracket,
    pub size: Expr,
    pub close: RightBracket,
}
