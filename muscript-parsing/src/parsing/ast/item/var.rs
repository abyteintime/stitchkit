use muscript_parsing_derive::{Parse, PredictiveParse};

use crate::{
    lexis::token::{Ident, LeftParen, RightParen, Semi},
    parsing::ast::Type,
};

keyword!(KVar = "var");

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemVar {
    pub var: KVar,
    pub editor: Option<VarEditor>,
    pub ty: Type,
    pub name: Ident,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct VarEditor {
    pub open: LeftParen,
    pub category: Option<Ident>,
    pub close: RightParen,
}
