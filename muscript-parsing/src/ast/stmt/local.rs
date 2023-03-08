use crate::{
    ast::Type,
    diagnostics,
    lexis::token::{Ident, Semi},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KLocal = "local");

#[derive(Debug, Clone, PredictiveParse)]
pub struct StmtLocal {
    pub local: KLocal,
    pub ty: Type,
    pub vars: Vec<Ident>,
    pub semi: Semi,
}

impl Parse for StmtLocal {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let local = parser.parse()?;
        let ty = parser.parse()?;
        let (vars, semi) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(&local, error, diagnostics::sets::VARIABLES)
        })?;
        Ok(Self {
            local,
            ty,
            vars,
            semi,
        })
    }
}
