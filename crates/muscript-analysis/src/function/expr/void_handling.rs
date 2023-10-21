//! Helper functions for implementing diagnostics that reference types.
//!
//! Expressions may produce registers with the `Void` type, which only appears as a result of other
//! errors. Thus, `Void` should never appear in actual error messages. The functions in this module
//! help with filtering out `Void` results.

use crate::ir::{Ir, RegisterId, Value};

pub fn registers_are_valid(ir: &Ir, registers: &[RegisterId]) -> bool {
    registers
        .iter()
        .all(|&register| !matches!(ir.register(register).value, Value::Void))
}
