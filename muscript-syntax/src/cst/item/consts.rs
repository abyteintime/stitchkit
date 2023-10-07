use muscript_syntax_derive::Spanned;

use crate::{
    cst::{Expr, KConst},
    token::{Assign, Ident, Semi},
    Parse, PredictiveParse,
};

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemConst {
    pub kconst: KConst,
    // TODO: Alter the error here somehow to say "constant name expected"
    pub name: Ident,
    pub equals: Assign,
    pub value: Expr,
    pub semi: Semi,
}
