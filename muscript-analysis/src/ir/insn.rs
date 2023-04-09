use crate::FunctionId;

use super::RegisterId;

/// [`Value`] represents an instruction that produces a value.
#[derive(Clone)]
pub enum Value {
    Void,

    // Primitives
    Bool(bool),
    Int(i32),
    Float(f32),

    // Objects
    None,

    // Calls
    CallFinal {
        function: FunctionId,
        arguments: Vec<RegisterId>,
    },
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
    Discard(RegisterId),
}

/// [`Terminator`] represents an instruction which ends the execution of a basic block.
///
/// Every basic block must end with a [`Terminator`]; this ensures the control flow forms an easily
/// digestible graph. Like [`Sink`]s, [`Terminator`]s do not produce any meaningful result.
#[derive(Clone, Default)]
pub enum Terminator {
    #[default]
    Unreachable,
    /// Return a value from the function.
    ///
    /// If a function is to return nothing (`void`), use this in conjunction with [`Value::Void`].
    Return(RegisterId),
}
