use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    ops::Deref,
};

use anyhow::Context;

#[derive(Debug, Clone, Copy)]
pub struct Deserializer<R> {
    stream_length: u64,
    stream_position: u64,
    stream: R,
}

impl<R> Deserializer<R> {
    pub fn stream_length(&self) -> u64 {
        self.stream_length
    }

    pub fn stream_position(&self) -> u64 {
        self.stream_position
    }

    pub fn read_bytes(&mut self, out_bytes: &mut [u8]) -> anyhow::Result<()>
    where
        R: Read,
    {
        self.stream
            .read_exact(out_bytes)
            .with_context(|| format!("at stream position {:08x}", self.stream_position))?;
        self.stream_position += out_bytes.len() as u64;
        Ok(())
    }

    pub fn read_to_end(&mut self, out_bytes: &mut Vec<u8>) -> anyhow::Result<()>
    where
        R: Read,
    {
        self.stream.read_to_end(out_bytes)?;
        self.stream_position = self.stream_length;
        Ok(())
    }

    pub fn seek(&mut self, whence: SeekFrom) -> anyhow::Result<u64>
    where
        R: Seek,
    {
        self.stream_position = self.stream.seek(whence)?;
        Ok(self.stream_position)
    }
}

impl<T> Deserializer<Cursor<T>>
where
    T: Deref<Target = [u8]>,
{
    pub fn from_buffer(buffer: T) -> Self {
        Self::from(Cursor::new(buffer))
    }
}

impl<R> Deserializer<R>
where
    R: Read + Seek,
{
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        let position = reader
            .stream_position()
            .context("cannot obtain current stream position")?;
        let stream_length = reader
            .seek(std::io::SeekFrom::End(0))
            .context("cannot obtain stream length")?;
        reader
            .seek(std::io::SeekFrom::Start(position))
            .context("cannot go back to previous stream position after obtaining its length")?;
        Ok(Self {
            stream_length,
            stream_position: position,
            stream: reader,
        })
    }
}

impl<T> From<Cursor<T>> for Deserializer<Cursor<T>>
where
    T: Deref<Target = [u8]>,
{
    fn from(cursor: Cursor<T>) -> Self {
        Self {
            stream_length: cursor.get_ref().len() as u64,
            stream_position: cursor.position(),
            stream: cursor,
        }
    }
}
