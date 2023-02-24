use thiserror::Error;

pub mod structure;
pub mod writer;

#[derive(Debug, Error)]
pub enum Error {
    #[error("formatting error: {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("when serializing the class name {0:?}, shifting the character {1} would result in invalid Unicode")]
    CharNotShiftable(String, char),
}
