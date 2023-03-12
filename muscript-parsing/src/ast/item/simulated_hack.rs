//! Somewhat of a hack to support parsing `simulated function` and `simulated state` predictively.

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{ast::KSimulated, lexis::token::Token, Parse, ParseStream, Parser, PredictiveParse};

use super::{ItemFunction, ItemState};

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemSimulated {
    pub simulated: KSimulated,
    pub item: SimulatedItem,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "simulated_item_error")]
pub enum SimulatedItem {
    Function(ItemFunction),
    State(ItemState),
}

fn simulated_item_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        "function or state item expected after `simulated`",
    )
    .with_label(Label::primary(
        token.span,
        "function or state expected here",
    ))
}
