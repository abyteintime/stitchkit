use std::io::Write;

use crate::binary::{Error, ErrorKind, ResultMapToBinaryErrorExt};

#[derive(Debug, Clone, Copy)]
pub struct Serializer<W> {
    stream: W,
}

impl<W> Serializer<W> {
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
    pub fn new(writer: W) -> Self {
        Self { stream: writer }
    }

    pub fn into_inner(self) -> W {
        self.stream
    }
}
