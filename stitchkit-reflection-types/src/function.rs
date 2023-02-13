#![allow(clippy::manual_strip)]

use std::num::{NonZeroU16, NonZeroU8};

use bitflags::bitflags;
use stitchkit_archive::name::ArchivedName;
use stitchkit_core::{serializable_bitflags, serializable_structure};

use crate::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub chunk: Chunk,
    /// Which VM opcode this function implements. This is the number in the `native(n)` specifier.
    pub native_index: Option<NonZeroU16>,
    /// The precedence this operator should take when parsing expressions. This is the number in the
    /// `operator(n)` specifier.
    pub infix_operator_precedence: Option<NonZeroU8>,
    pub function_flags: FunctionFlags,
    pub name: ArchivedName,
}

serializable_structure! {
    type Function {
        chunk,
        native_index,
        infix_operator_precedence,
        function_flags,
        name,
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FunctionFlags: u32 {
        const FINAL               = 0x00000001;
        const BYTECODE            = 0x00000002;
        const PREFIX_OPERATOR     = 0x00000010;
        const SIMULATED           = 0x00000100;
        const NATIVE              = 0x00000400;
        const EVENT               = 0x00000800;
        const OPERATOR            = 0x00001000;
        const STATIC              = 0x00002000;
        const HAS_OPTIONAL_PARAMS = 0x00004000;
        const PUBLIC              = 0x00020000;
        const PRIVATE             = 0x00040000;
        const HAS_OUT_PARAMS      = 0x00400000;
    }
}

serializable_bitflags!(FunctionFlags);
