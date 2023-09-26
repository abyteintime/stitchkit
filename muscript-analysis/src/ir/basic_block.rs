use std::borrow::Cow;

use muscript_syntax::lexis::token::TokenSpan;

use crate::ir::Terminator;

use super::NodeId;

/// A basic block represents a continuous sequence of [`Node`]s ended with a [`Terminator`].
///
/// [`Node`]: super::Node
#[derive(Clone)]
pub struct BasicBlock {
    pub label: Cow<'static, str>,
    pub flow: Vec<NodeId>,
    pub terminator: Terminator,
    pub span: TokenSpan,
}

impl BasicBlock {
    pub fn new(label: impl Into<Cow<'static, str>>, span: TokenSpan) -> Self {
        Self {
            label: label.into(),
            flow: vec![],
            terminator: Terminator::default(),
            span,
        }
    }
}
