use std::fmt::{self, Display};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorKind {
    #[error("deserialization error")]
    Deserialize,
    #[error("serialization error")]
    Serialize,
}

impl ErrorKind {
    pub fn make(self, message: impl Into<String>) -> Error {
        Error {
            kind: self,
            context_stack: vec![message.into()],
        }
    }
}

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

pub trait ResultContextExt {
    fn context(self, text: &str) -> Self;
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

pub trait ResultMapToBinaryErrorExt<T> {
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
