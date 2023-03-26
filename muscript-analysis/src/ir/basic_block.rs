use crate::ir::Terminator;

use super::NodeId;

/// A basic block represents a continuous sequence of [`Node`]s ended with a [`Terminator`].
///
/// [`Node`]: super::Node
#[derive(Clone)]
pub struct BasicBlock {
    pub flow: Vec<NodeId>,
    pub terminator: Terminator,
}
