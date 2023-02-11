pub mod structure;

use std::io::{Cursor, Read};

use anyhow::Context;
use uuid::Uuid;

pub trait Deserialize: Sized {
    fn deserialize(reader: impl Read) -> anyhow::Result<Self>;
}

impl Deserialize for u8 {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Deserialize for u16 {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

impl Deserialize for u32 {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

impl Deserialize for u64 {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl Deserialize for Uuid {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;
        Ok(Uuid::from_bytes(buf))
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let len = reader
            .deserialize::<u32>()
            .context("cannot read array length")? as usize;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(reader.deserialize().with_context(|| {
                format!("cannot deserialize array field {i} (array of length {len})")
            })?);
        }
        Ok(vec)
    }
}

pub trait ReadExt {
    fn deserialize<T>(&mut self) -> anyhow::Result<T>
    where
        T: Deserialize;
}

impl<R> ReadExt for R
where
    R: Read,
{
    fn deserialize<T>(&mut self) -> anyhow::Result<T>
    where
        T: Deserialize,
    {
        T::deserialize(self)
    }
}

pub fn deserialize<T>(slice: &[u8]) -> anyhow::Result<(T, &[u8])>
where
    T: Deserialize,
{
    let mut cursor = Cursor::new(slice);
    let result = T::deserialize(&mut cursor)?;
    let position = cursor.position();
    Ok((result, &slice[position as usize..]))
}
