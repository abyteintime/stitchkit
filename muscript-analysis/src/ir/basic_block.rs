use std::borrow::Cow;

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
}

impl BasicBlock {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            flow: vec![],
            terminator: Terminator::default(),
        }
    }
}
