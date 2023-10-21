use std::fmt::{self, Display};

use thiserror::Error;

/// Binary (de)serialization error kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorKind {
    #[error("deserialization error")]
    Deserialize,
    #[error("serialization error")]
    Serialize,
}

impl ErrorKind {
    /// Creates an error of this kind with the given message.
    pub fn make(self, message: impl Into<String>) -> Error {
        Error {
            kind: self,
            context_stack: vec![message.into()],
        }
    }
}

/// Binary (de)serialization error.
///
/// Since tracking down bugs in binary deserialization can be quite hard without enough contextual
/// information, this error contains an internal _context stack_, not unlike what's done with
/// [`anyhow`]'s errors. Messages can be appended onto this stack using [`ResultContextExt`].
///
/// [`anyhow`]: https://crates.io/crate/anyhow
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    context_stack: Vec<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.kind)?;
        for (i, context) in self.context_stack.iter().rev().enumerate() {
            // TODO: Maybe something prettier, more akin to anyhow's Indented writer?
            write!(f, "{i:5}: {context}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

/// Extensions to [`Result<T, E>`] that allow attaching additional context.
///
/// This is implemented on `Result<T, `[`struct@Error`]`>`, allowing to extend the
/// [`struct@Error`]'s context stack.
pub trait ResultContextExt {
    /// Extends an [`Err`] variant with a context message.
    fn context(self, text: &str) -> Self;

    /// Extends an [`Err`] variant with a lazily evaluated context message. The message is only
    /// evaluated if the result is [`Err`].
    fn with_context(self, text: impl FnOnce() -> String) -> Self;
}

impl<T> ResultContextExt for Result<T, Error> {
    fn context(self, text: &str) -> Self {
        self.with_context(|| text.to_string())
    }

    fn with_context(self, text: impl FnOnce() -> String) -> Self {
        self.map_err(|mut error| {
            error.context_stack.push(text());
            error
        })
    }
}

/// Extension to [`Result<T, E>`] that allows mapping any result to a `Result<T, `[`struct@Error`]`>`
/// by stringifying it.
pub trait ResultMapToBinaryErrorExt<T> {
    /// Maps the [`Err`] variant to an [`struct@Error`].
    fn map_err_to_binary_error(self, kind: ErrorKind) -> Result<T, Error>;
}

impl<T, E> ResultMapToBinaryErrorExt<T> for Result<T, E>
where
    E: Display,
{
    fn map_err_to_binary_error(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|error| kind.make(error.to_string()))
    }
}
