use muscript_foundation::source::SourceFileSet;

pub mod class;
mod diagnostics;
mod environment;
pub mod function;
mod package;
pub mod partition;
mod source;
pub mod type_system;

pub use environment::*;
pub use package::*;
pub use source::*;

/// Full compiler state.
pub struct Compiler<'a> {
    pub sources: &'a SourceFileSet,
    pub env: &'a mut Environment,
    pub input: &'a dyn CompilerInput,
}

/// Compilation failed irrecoverably.
///
/// No artifacts were produced; the environment can be checked to obtain detailed diagnostics on
/// why the error occurred.
pub struct CompileError;
