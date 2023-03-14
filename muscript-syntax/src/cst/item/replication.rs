use crate::{
    cst::{KIf, ParenExpr},
    lexis::token::{Ident, LeftBrace, RightBrace, Semi},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword!(KReplication = "replication");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemReplication {
    pub replication: KReplication,
    pub open: LeftBrace,
    pub conditions: Vec<RepCondition>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct RepCondition {
    pub kif: KIf,
    pub cond: ParenExpr,
    pub vars: Vec<Ident>,
    pub semi: Semi,
}

impl Parse for ItemReplication {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Ok(Self {
            replication: parser.parse()?,
            open: parser.parse()?,
            conditions: parser.parse_greedy_list()?,
            close: parser.parse()?,
        })
    }
}

impl Parse for RepCondition {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let kif = parser.parse()?;
        let cond = parser.parse()?;
        let (vars, semi) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &kif,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `;` after variable list",
                    missing_right_label: "this replication condition does not have a `;`",
                    missing_comma: "`,` or `;` expected after variable",
                    missing_comma_open: "in this replication condition",
                    missing_comma_token: "this was expected to continue or end the variable list",
                    missing_comma_note:
                        "note: variables in replication conditions are separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            kif,
            cond,
            vars,
            semi,
        })
    }
}
