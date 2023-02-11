pub mod macros;

use std::io::{Cursor, Read};

use anyhow::Context;
use uuid::Uuid;

pub trait Deserialize: Sized {
    fn deserialize(reader: impl Read) -> anyhow::Result<Self>;
}

macro_rules! deserialize_primitive_le {
    ($T:ty) => {
        impl Deserialize for $T {
            fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
                let mut buf = [0; std::mem::size_of::<$T>()];
                reader.read_exact(&mut buf)?;
                Ok(<$T>::from_le_bytes(buf))
            }
        }
    };
}

deserialize_primitive_le!(u8);
deserialize_primitive_le!(u16);
deserialize_primitive_le!(u32);
deserialize_primitive_le!(u64);

deserialize_primitive_le!(i8);
deserialize_primitive_le!(i16);
deserialize_primitive_le!(i32);
deserialize_primitive_le!(i64);

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
