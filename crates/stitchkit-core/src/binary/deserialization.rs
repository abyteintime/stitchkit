mod deserializer;

pub use deserializer::*;

use std::{
    io::Read,
    num::{
        NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64,
        NonZeroU8,
    },
};

use uuid::Uuid;

use crate::binary::error::ResultContextExt;

use super::{Error, ErrorKind};

/// Implemented by everything deserializable from bytes.
pub trait Deserialize: Sized {
    /// Deserializes the value from bytes.
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error>;
}

impl Deserialize for () {
    fn deserialize(_: &mut Deserializer<impl Read>) -> Result<Self, Error> {
        Ok(())
    }
}

macro_rules! deserialize_primitive_le {
    ($T:ty) => {
        impl Deserialize for $T {
            fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
                let mut buf = [0; std::mem::size_of::<$T>()];
                deserializer.read_bytes(&mut buf)?;
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

deserialize_primitive_le!(f32);
deserialize_primitive_le!(f64);

macro_rules! deserialize_nonzero_primitive_le {
    ($Underlying:ty, $NonZero:ty) => {
        impl Deserialize for $NonZero {
            fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
                let num = deserializer.deserialize::<$Underlying>()?;
                <$NonZero>::new(num).ok_or_else(|| {
                    ErrorKind::Deserialize.make("non-zero value expected but got zero")
                })
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
            fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
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

/// `Vec<T>` is serialized as a `u32` size followed by the vector's elements.
impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
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

impl Deserialize for Uuid {
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
        let mut buf = [0; 16];
        deserializer.read_bytes(&mut buf)?;
        Ok(Uuid::from_bytes_le(buf))
    }
}

impl<R> Deserializer<R> {
    /// Convenience function that deserializes a type implementing [`Deserialize`] from the current
    /// stream position.
    pub fn deserialize<T>(&mut self) -> Result<T, Error>
    where
        R: Read,
        T: Deserialize,
    {
        T::deserialize(self)
    }
}

/// Convenience function that deserializes a type implementing [`Deserialize`] from a buffer.
pub fn deserialize<T>(buffer: &[u8]) -> Result<T, Error>
where
    T: Deserialize,
{
    T::deserialize(&mut Deserializer::from_buffer(buffer))
}
