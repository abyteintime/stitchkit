use crate::{
    ast::{Expr, KConst},
    lexis::token::{Assign, Ident, Semi},
    Parse, PredictiveParse,
};

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemConst {
    pub kconst: KConst,
    // TODO: Alter the error here somehow to say "constant name expected"
    pub name: Ident,
    pub equals: Assign,
    pub value: Expr,
    pub semi: Semi,
}
