#[macro_use]
pub mod token;

mod lexer;
// NOTE: Preprocessor is currently disabled and will be implemented as a separate pass after lexis.
// Possibly in a separate crate.
// pub mod preprocessor;
mod token_stream;

pub use lexer::*;
pub use token_stream::*;
