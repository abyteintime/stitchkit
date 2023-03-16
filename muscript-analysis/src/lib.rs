pub mod class;
mod environment;
mod package;
mod source;

pub use environment::*;
pub use package::*;
pub use source::*;

/// Compilation failed irrecoverably.
///
/// No artifacts were produced; the environment can be checked to obtain detailed diagnostics on
/// why the error occurred.
pub struct CompileError;
