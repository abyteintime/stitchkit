use std::io::Write;

use crate::binary::{Error, ErrorKind, ResultMapToBinaryErrorExt};

/// Serialization state.
#[derive(Debug, Clone, Copy)]
pub struct Serializer<W> {
    stream: W,
}

impl<W> Serializer<W> {
    /// Writes the specified bytes to the output stream.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error>
    where
        W: Write,
    {
        self.stream
            .write_all(bytes)
            .map_err_to_binary_error(ErrorKind::Serialize)
    }
}

impl<W> Serializer<W> {
    /// Creates a new serializer from the given output stream.
    pub fn new(writer: W) -> Self {
        Self { stream: writer }
    }

    /// Consumes the serializer state and returns the underlying output stream.
    pub fn into_inner(self) -> W {
        self.stream
    }
}
