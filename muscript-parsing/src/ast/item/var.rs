use crate::{
    ast::Type,
    lexis::{
        token::{Ident, LeftParen, RightParen, Semi},
        TokenStream,
    },
    list::DelimitedListDiagnostics,
    Parse, ParseError, Parser, PredictiveParse,
};

keyword!(KVar = "var");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemVar {
    pub var: KVar,
    pub editor: Option<VarEditor>,
    pub ty: Type,
    pub names: Vec<Ident>,
    pub semi: Semi,
}

impl Parse for ItemVar {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let var = parser.parse()?;
        let editor = parser.parse()?;
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
                    missing_comma_token: "this was expected to continue or close the variable declaration",
                    missing_comma_note:
                        "note: multiple variable names in one `var` must be separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            var,
            editor,
            ty,
            names,
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
