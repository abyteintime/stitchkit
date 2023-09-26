//! Somewhat of a hack to support parsing `simulated function` and `simulated state` predictively.

use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    cst::KSimulated,
    lexis::token::{AnyToken, Token},
    Parse, ParseStream, Parser, PredictiveParse,
};

use super::{ItemFunction, ItemState};

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemSimulated {
    pub simulated: KSimulated,
    pub item: SimulatedItem,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "simulated_item_error")]
pub enum SimulatedItem {
    Function(ItemFunction),
    State(ItemState),
}

fn simulated_item_error(_: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("function or state item expected after `simulated`")
        .with_label(Label::primary(token, "function or state expected here"))
}
