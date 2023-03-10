use crate::{
    ast::{Cond, KIf},
    lexis::token::{Ident, LeftBrace, RightBrace, Semi},
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
    pub cond: Cond,
    pub vars: Vec<Ident>,
}

//

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct RepVar {
    pub name: Ident,
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
        Ok(Self {
            kif: parser.parse()?,
            cond: parser.parse()?,
            vars: parser.parse_greedy_list()?,
        })
    }
}
