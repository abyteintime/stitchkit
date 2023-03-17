use muscript_syntax_derive::Spanned;

use crate::{
    cst::{Type, VarDef},
    diagnostics,
    lexis::token::Semi,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KLocal = "local");

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct StmtLocal {
    pub local: KLocal,
    pub ty: Type,
    pub vars: Vec<VarDef>,
    pub semi: Semi,
}

impl Parse for StmtLocal {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let local = parser.parse()?;
        let ty = parser.parse()?;
        let (vars, semi) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(&local, error, diagnostics::sets::VARIABLES)
        })?;
        Ok(Self {
            local,
            ty,
            vars,
            semi,
        })
    }
}
