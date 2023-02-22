use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct Serializer<W> {
    stream: W,
}

impl<W> Serializer<W> {
    pub fn write_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()>
    where
        W: Write,
    {
        self.stream.write_all(bytes)?;
        Ok(())
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
