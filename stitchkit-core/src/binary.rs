pub mod macros;

use std::{
    io::{Cursor, Read, Seek},
    num::{
        NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64,
        NonZeroU8,
    },
    ops::{Deref, DerefMut},
};

use anyhow::{anyhow, Context};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Deserializer<R> {
    /// Hint about the length of the stream. This is used by some deserializers to know how much
    /// input to consume.
    pub stream_length: usize,
    pub stream: R,
}

impl<R> Deref for Deserializer<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<R> DerefMut for Deserializer<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

pub trait Deserialize: Sized {
    fn deserialize(deserializer: Deserializer<impl Read>) -> anyhow::Result<Self>;
}

impl Deserialize for () {
    fn deserialize(_: Deserializer<impl Read>) -> anyhow::Result<Self> {
        Ok(())
    }
}

macro_rules! deserialize_primitive_le {
    ($T:ty) => {
        impl Deserialize for $T {
            fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
                let mut buf = [0; std::mem::size_of::<$T>()];
                deserializer.read_exact(&mut buf)?;
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

macro_rules! deserialize_nonzero_primitive_le {
    ($Underlying:ty, $NonZero:ty) => {
        impl Deserialize for $NonZero {
            fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
                let num = deserializer.deserialize::<$Underlying>()?;
                <$NonZero>::new(num).ok_or_else(|| anyhow!("non-zero value expected but got zero"))
            }
        }
    };
}

deserialize_nonzero_primitive_le!(u8, NonZeroU8);
deserialize_nonzero_primitive_le!(u16, NonZeroU16);
deserialize_nonzero_primitive_le!(u32, NonZeroU32);
deserialize_nonzero_primitive_le!(u64, NonZeroU64);

deserialize_nonzero_primitive_le!(i8, NonZeroI8);
deserialize_nonzero_primitive_le!(i16, NonZeroI16);
deserialize_nonzero_primitive_le!(i32, NonZeroI32);
deserialize_nonzero_primitive_le!(i64, NonZeroI64);

macro_rules! deserialize_optional_nonzero_primitive_le {
    ($Underlying:ty, $NonZero:ty) => {
        impl Deserialize for Option<$NonZero> {
            fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
                let num = deserializer.deserialize::<$Underlying>()?;
                Ok(<$NonZero>::new(num))
            }
        }
    };
}

deserialize_optional_nonzero_primitive_le!(u8, NonZeroU8);
deserialize_optional_nonzero_primitive_le!(u16, NonZeroU16);
deserialize_optional_nonzero_primitive_le!(u32, NonZeroU32);
deserialize_optional_nonzero_primitive_le!(u64, NonZeroU64);

deserialize_optional_nonzero_primitive_le!(i8, NonZeroI8);
deserialize_optional_nonzero_primitive_le!(i16, NonZeroI16);
deserialize_optional_nonzero_primitive_le!(i32, NonZeroI32);
deserialize_optional_nonzero_primitive_le!(i64, NonZeroI64);

impl Deserialize for Uuid {
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let mut buf = [0; 16];
        deserializer.read_exact(&mut buf)?;
        Ok(Uuid::from_bytes(buf))
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let len = deserializer
            .deserialize::<u32>()
            .context("cannot read array length")? as usize;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(deserializer.deserialize().with_context(|| {
                format!("cannot deserialize array field {i} (array of length {len})")
            })?);
        }
        Ok(vec)
    }
}

#[derive(Debug, Clone)]
pub struct TrailingData(pub Vec<u8>);

impl Deserialize for TrailingData {
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let mut buffer = vec![];
        deserializer
            .read_to_end(&mut buffer)
            .context("cannot deserialize trailing data")?;
        Ok(Self(buffer))
    }
}

impl Deref for TrailingData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R> Deserializer<R> {
    pub fn as_mut(&mut self) -> Deserializer<&mut R> {
        Deserializer {
            stream: &mut self.stream,
            stream_length: self.stream_length,
        }
    }

    pub fn deserialize<T>(&mut self) -> anyhow::Result<T>
    where
        R: Read,
        T: Deserialize,
    {
        T::deserialize(self.as_mut())
    }
}

pub fn deserialize<T>(buffer: &[u8]) -> anyhow::Result<T>
where
    T: Deserialize,
{
    T::deserialize(Deserializer::from_buffer(buffer))
}

impl<T> From<Cursor<T>> for Deserializer<Cursor<T>>
where
    T: Deref<Target = [u8]>,
{
    fn from(cursor: Cursor<T>) -> Self {
        Self {
            stream_length: cursor.get_ref().len(),
            stream: cursor,
        }
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
    pub fn new(mut deserializer: R) -> anyhow::Result<Self> {
        let position = deserializer
            .stream_position()
            .context("cannot obtain current stream position")?;
        let stream_length = deserializer
            .seek(std::io::SeekFrom::End(0))
            .context("cannot obtain stream length")?;
        deserializer
            .seek(std::io::SeekFrom::Start(position))
            .context("cannot go back to previous stream position after obtaining its length")?;
        Ok(Self {
            stream_length: stream_length as usize,
            stream: deserializer,
        })
    }
}
