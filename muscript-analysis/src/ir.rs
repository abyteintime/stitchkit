mod basic_block;
pub mod dump;
mod insn;

use std::borrow::Cow;

use muscript_foundation::source::Span;

use crate::{TypeId, VarId};

pub use basic_block::*;
pub use insn::*;

/// Represents the IR of a chunk (a function, or some other unit of execution.)
#[derive(Clone)]
pub struct Ir {
    pub return_ty: TypeId,
    /// Local variables declared in the chunk.
    pub locals: Vec<VarId>,
    /// The first `param_count` locals are treated as the chunk's parameters.
    pub param_count: u16,

    pub nodes: Vec<Node>,
    /// The first basic block in the function is treated as its entry point. Further blocks must
    /// be reached via this block.
    pub basic_blocks: Vec<BasicBlock>,
}

/// Unique ID of a [`Node`] within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

/// Represents an execution node inside of a function.
///
/// Since nodes may be referenced between basic blocks, they belong to a registry inside of [`Ir`].
///
/// The order of evaluation is defined by the [`Sink`] nodes inside the functions; the execution of
/// [`Register`] nodes is driven by [`Sink`]s that use them. The reason why they're grouped into one
/// type is so that inserting a sink before a certain register is possible and easy to do.
#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    /// The source span that produced this node.
    pub span: Span,
}

/// The kind of an execution node inside of a function.
///
/// See [`Node`] for more details.
#[derive(Clone)]
pub enum NodeKind {
    Register(Register),
    Sink(Sink),
}

/// An register represents a single [`Value`] produced in a basic block.
///
/// Registers can only be assigned once, but may be reused. This representation actually conflicts
/// with UnrealScript bytecode, which is a more AST-like structure, where a single value may only be
/// used once.
///
/// MuScript works around this by marking registers which are reused and lowering them
/// into operations on local variables. As such, it's much more efficient to only use each register
/// once, because then they can be inlined into their sinks.
#[derive(Clone)]
pub struct Register {
    /// Name for debugging purposes.
    ///
    /// This name is also used when lowering reused registers into variables, so it's best to have
    /// it be somewhat meaningful to the end user.
    pub name: Cow<'static, str>,
    pub value: Value,
}
