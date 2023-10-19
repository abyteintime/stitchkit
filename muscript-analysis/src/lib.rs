pub mod class;
mod diagnostics;
mod environment;
pub mod function;
pub mod ir;
mod package;
pub mod partition;
mod source;
pub mod type_system;

pub use environment::*;
use muscript_lexer::sources::OwnedSources;
pub use package::*;
pub use source::*;

/// Full compiler state.
pub struct Compiler<'a> {
    pub sources: &'a mut OwnedSources<'a>,
    pub env: &'a mut Environment,
    pub input: &'a dyn CompilerInput,
}

/// Compilation failed irrecoverably.
///
/// No artifacts were produced; the environment can be checked to obtain detailed diagnostics on
/// why the error occurred.
pub struct CompileError;
