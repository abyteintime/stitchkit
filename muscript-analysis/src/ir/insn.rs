use super::NodeId;

/// [`Value`] represents an instruction that produces a value.
#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f32),
}

/// [`Sink`] represents a side-effectful instruction that does not produce a meaningful result.
///
/// [`Sink`]s integrate tightly with [`Value`]s. A value on its own does not actually do anything;
/// it has to be sunk into an `Sink` to be evaluated. As such, `Sink`s define the evaluation order
/// of values.
///
/// [`BasicBlock`]: crate::BasicBlock
#[derive(Clone)]
pub enum Sink {
    /// Evaluates the value from the given register effectfully and discards its result.
    Discard(NodeId),
}

/// [`Terminator`] represents an instruction which ends the execution of a basic block.
///
/// Every basic block must end with a [`Terminator`]; this ensures the control flow forms an easily
/// digestible graph. Like [`Sink`]s, [`Terminator`]s do not produce any meaningful result.
#[derive(Clone)]
pub enum Terminator {
    Return,
}
