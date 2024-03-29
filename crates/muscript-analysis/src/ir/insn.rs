use crate::{ClassId, FunctionId, VarId};

use super::{BasicBlockId, Ir, RegisterId};

/// [`Value`] represents an instruction that produces a value.
#[derive(Clone)]
pub enum Value {
    /// No value, literally. Not even `none`.
    ///
    /// Emitted when the compiler encounters an error and needs a placeholder.
    Void,

    /// # Literals

    /// Constant `Bool` value.
    Bool(bool),
    /// Constant `Byte` value.
    Byte(u8),
    /// Constant `Int` value.
    Int(i32),
    /// Constant `Float` value.
    Float(f32),
    /// Constant `String` value.
    String(String),
    /// Constant `Name` value.
    Name(String),

    /// # Places

    /// Reference to a local variable.
    Local(VarId),
    /// Reference to a field on `self`.
    Field(VarId),

    /// # Casts
    PrimitiveCast {
        kind: PrimitiveCast,
        value: RegisterId,
    },

    /// # Arrays
    Len(RegisterId),
    Index {
        array: RegisterId,
        index: RegisterId,
    },

    /// # Objects

    /// The `none` literal.
    None,
    /// What's known as `self` in UnrealScript; unfortunately we can't use that identifier since
    /// it's reserved in Rust.
    This,
    /// Reference to an object.
    Object {
        class: ClassId,
        package: String,
        name: String,
    },
    /// Performs `action` with `self` changed to something else.
    In {
        /// The object to use as `self` for `action`. Note that passing `This` here is redundant
        /// and will be optimized out.
        context: RegisterId,
        action: RegisterId,
    },

    /// # Functions

    /// Call precisely the given `function` with the given `arguments`. No dynamic dispatch is
    /// performed, so this is the fastest way to call a function.
    CallFinal {
        function: FunctionId,
        arguments: Vec<RegisterId>,
    },
    /// Signal that an argument in a function call was omitted and its default value should be used.
    Default,
}

/// Primitive casts supported by the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrimitiveCast {
    InterfaceToObject = 54,
    InterfaceToString = 55,
    InterfaceToBool = 56,
    RotatorToVector = 57,
    ByteToInt = 58,
    ByteToBool = 59,
    ByteToFloat = 60,
    IntToByte = 61,
    IntToBool = 62,
    IntToFloat = 63,
    BoolToByte = 64,
    BoolToInt = 65,
    BoolToFloat = 66,
    FloatToByte = 67,
    FloatToInt = 68,
    FloatToBool = 69,
    // ObjectToInterface casts are not supported here because they also encode the interface to
    // cast to, which would prevent this enum from being C-like. Instead it's a separate Value variant.
    // ObjectToInterface = 70,
    ObjectToBool = 71,
    NameToBool = 72,
    StringToByte = 73,
    StringToInt = 74,
    StringToBool = 75,
    StringToFloat = 76,
    StringToVector = 77,
    StringToRotator = 78,
    VectorToBool = 79,
    VectorToRotator = 80,
    RotatorToBool = 81,
    ByteToString = 82,
    IntToString = 83,
    BoolToString = 84,
    FloatToString = 85,
    ObjectToString = 86,
    NameToString = 87,
    VectorToString = 88,
    RotatorToString = 89,
    DelegateToString = 90,
    StringToName = 96,
}

/// [`Sink`] represents a side-effectful instruction that does not produce a meaningful result.
///
/// [`Sink`]s integrate tightly with [`Value`]s. A value on its own does not actually do anything;
/// it has to be sunk into an `Sink` to be evaluated. As such, `Sink`s define the evaluation order
/// of values.
#[derive(Clone)]
pub enum Sink {
    /// Evaluates the value from the given register effectfully and discards its result.
    Discard(RegisterId),

    /// Stores the provided rvalue in the lvalue produced by the given register.
    Store(RegisterId, RegisterId),
}

/// [`Terminator`] represents an instruction which ends the execution of a basic block.
///
/// Every basic block must end with a [`Terminator`]; this ensures the control flow forms an easily
/// digestible graph. Like [`Sink`]s, [`Terminator`]s do not produce any meaningful result.
#[derive(Clone, Default)]
pub enum Terminator {
    /// Block is unreachable and can be removed during optimization.
    #[default]
    Unreachable,

    /// Unconditionally go to another block after the current one's done executing.
    Goto(BasicBlockId),
    /// Conditionally go to one of two blocks after the current one's done executing.
    GotoIf {
        condition: RegisterId,
        if_true: BasicBlockId,
        if_false: BasicBlockId,
    },

    /// Return a value from the function.
    ///
    /// If a function is to return nothing (`void`), use this in conjunction with [`Value::Void`].
    Return(RegisterId),
}

impl Ir {
    /// Returns whether the given register is a place (something that can be assigned to or passed
    /// to `out` parameters.)
    ///
    /// Currently this does not consider `const`ness; MuScript just blindly ignores `const` and
    /// allows you to assign everywhere.
    pub fn is_place(&self, register: RegisterId) -> bool {
        match &self.register(register).value {
            Value::Local(_) | Value::Field(_) | Value::Index { .. } => true,
            Value::In { context: _, action } => self.is_place(*action),
            _ => false,
        }
    }
}
