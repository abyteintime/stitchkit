mod basic_block;
pub mod dump;
mod insn;

use std::borrow::Cow;

use muscript_foundation::source::Span;

use crate::{TypeId, VarId};

pub use basic_block::*;
pub use insn::*;

/// Represents the IR of a chunk (a function, or some other unit of execution.)
#[derive(Clone, Default)]
pub struct Ir {
    /// Local variables declared in the chunk.
    pub locals: Vec<VarId>,

    pub nodes: Vec<Node>,
    /// The first basic block in the function is treated as its entry point. Further blocks must
    /// be reached via this block.
    pub basic_blocks: Vec<BasicBlock>,
}

/// Unique ID of a [`Node`] within an [`Ir`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

/// Unique ID of a [`Register`] [`Node`] within an [`Ir`]. This can be thought of as a more
/// specialized version of [`NodeId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterId(u32);

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
    pub ty: TypeId,
    pub value: Value,
}

/// Unique ID of a [`BasicBlock`] within an [`Ir`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(u32);

impl Ir {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_local(&mut self, var_id: VarId) {
        self.locals.push(var_id);
    }

    #[must_use]
    pub fn create_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }

    #[must_use]
    pub fn create_register(
        &mut self,
        span: Span,
        name: impl Into<Cow<'static, str>>,
        ty: TypeId,
        value: Value,
    ) -> RegisterId {
        RegisterId(
            self.create_node(Node {
                kind: NodeKind::Register(Register {
                    name: name.into(),
                    ty,
                    value,
                }),
                span,
            })
            .0,
        )
    }

    #[must_use]
    pub fn create_sink(&mut self, span: Span, sink: Sink) -> NodeId {
        self.create_node(Node {
            kind: NodeKind::Sink(sink),
            span,
        })
    }

    #[must_use]
    pub fn create_basic_block(&mut self, basic_block: BasicBlock) -> BasicBlockId {
        let id = BasicBlockId(self.basic_blocks.len() as u32);
        self.basic_blocks.push(basic_block);
        id
    }

    pub fn basic_block(&self, basic_block_id: BasicBlockId) -> &BasicBlock {
        &self.basic_blocks[basic_block_id.0 as usize]
    }

    pub fn basic_block_mut(&mut self, basic_block_id: BasicBlockId) -> &mut BasicBlock {
        &mut self.basic_blocks[basic_block_id.0 as usize]
    }

    pub fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id.0 as usize]
    }

    pub fn register(&self, register_id: RegisterId) -> &Register {
        match &self.node(register_id.into()).kind {
            NodeKind::Register(register) => register,
            NodeKind::Sink(_) => unreachable!("RegisterId must point to a register"),
        }
    }
}

impl NodeId {
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl From<RegisterId> for NodeId {
    fn from(value: RegisterId) -> Self {
        NodeId(value.0)
    }
}
