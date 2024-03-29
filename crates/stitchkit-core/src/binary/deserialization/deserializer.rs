use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    ops::Deref,
};

use crate::binary::{Error, ErrorKind, ResultContextExt, ResultMapToBinaryErrorExt};

/// Deserialization state.
#[derive(Debug, Clone, Copy)]
pub struct Deserializer<R> {
    stream_len: u64,
    stream_position: u64,
    stream: R,
}

impl<R> Deserializer<R> {
    /// Returns the full length of the input stream.
    pub fn stream_len(&self) -> u64 {
        self.stream_len
    }

    /// Returns the current position in the input stream.
    pub fn stream_position(&self) -> u64 {
        self.stream_position
    }

    /// Reads `out_bytes.len()` bytes from the output stream, returning an error if the data could
    /// not be read.
    pub fn read_bytes(&mut self, out_bytes: &mut [u8]) -> Result<(), Error>
    where
        R: Read,
    {
        self.stream
            .read_exact(out_bytes)
            .map_err_to_binary_error(ErrorKind::Deserialize)
            .with_context(|| format!("at stream position {:08x}", self.stream_position))?;
        self.stream_position += out_bytes.len() as u64;
        Ok(())
    }

    /// Reads the entire rest of the stream into `out_bytes`.
    pub fn read_to_end(&mut self, out_bytes: &mut Vec<u8>) -> Result<(), Error>
    where
        R: Read,
    {
        self.stream
            .read_to_end(out_bytes)
            .map_err_to_binary_error(ErrorKind::Deserialize)?;
        self.stream_position = self.stream_len;
        Ok(())
    }

    /// Seeks to the specified position within the stream.
    pub fn seek(&mut self, whence: SeekFrom) -> Result<u64, Error>
    where
        R: Seek,
    {
        self.stream_position = self
            .stream
            .seek(whence)
            .map_err_to_binary_error(ErrorKind::Deserialize)?;
        Ok(self.stream_position)
    }
}

impl<T> Deserializer<Cursor<T>>
where
    T: Deref<Target = [u8]>,
{
    /// Creates a new deserializer from a buffer. This is generally faster than using
    /// [`Deserializer::new`] because it avoids seeking.
    pub fn from_buffer(buffer: T) -> Self {
        Self::from(Cursor::new(buffer))
    }
}

impl<R> Deserializer<R>
where
    R: Read + Seek,
{
    /// Creates a new deserializer from an arbitrary reader.
    ///
    /// This will perform three seeks to establish the stream's length; whenever possible, prefer
    /// reading your stream to an in-memory buffer and using [`Deserializer::from_buffer`] instead.
    pub fn new(mut reader: R) -> Result<Self, Error> {
        let position = reader
            .stream_position()
            .map_err_to_binary_error(ErrorKind::Deserialize)
            .context("cannot obtain current stream position")?;
        let stream_length = reader
            .seek(std::io::SeekFrom::End(0))
            .map_err_to_binary_error(ErrorKind::Deserialize)
            .context("cannot obtain stream length")?;
        reader
            .seek(std::io::SeekFrom::Start(position))
            .map_err_to_binary_error(ErrorKind::Deserialize)
            .context("cannot go back to previous stream position after obtaining its length")?;
        Ok(Self {
            stream_len: stream_length,
            stream_position: position,
            stream: reader,
        })
    }
}

/// Create a stream from a [`Cursor<T>`]. This is the same as [`Deserializer::from_buffer`].
impl<T> From<Cursor<T>> for Deserializer<Cursor<T>>
where
    T: Deref<Target = [u8]>,
{
    fn from(cursor: Cursor<T>) -> Self {
        Self {
            stream_len: cursor.get_ref().len() as u64,
            stream_position: cursor.position(),
            stream: cursor,
        }
    }
}
