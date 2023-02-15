pub mod macros;

use std::{
    io::{Cursor, Read},
    num::{
        NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64,
        NonZeroU8,
    },
    ops::Deref,
};

use anyhow::{anyhow, Context};
use uuid::Uuid;

pub trait Deserialize: Sized {
    fn deserialize(reader: impl Read) -> anyhow::Result<Self>;
}

impl Deserialize for () {
    fn deserialize(_: impl Read) -> anyhow::Result<Self> {
        Ok(())
    }
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

macro_rules! deserialize_nonzero_primitive_le {
    ($Underlying:ty, $NonZero:ty) => {
        impl Deserialize for $NonZero {
            fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
                let num = reader.deserialize::<$Underlying>()?;
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
            fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
                let num = reader.deserialize::<$Underlying>()?;
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

#[derive(Debug, Clone)]
pub struct TrailingData(pub Vec<u8>);

impl Deserialize for TrailingData {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let mut buffer = vec![];
        reader
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

pub fn deserialize<T>(buffer: &[u8]) -> anyhow::Result<T>
where
    T: Deserialize,
{
    T::deserialize(Cursor::new(buffer))
}
