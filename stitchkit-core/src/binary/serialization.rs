mod serializer;

pub use serializer::*;
use uuid::Uuid;

use std::{
    io::{Cursor, Write},
    num::{
        NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64,
        NonZeroU8,
    },
};

use super::{Error, ResultContextExt};

pub trait Serialize: Sized {
    fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error>;
}

impl Serialize for () {
    fn serialize(&self, _: &mut Serializer<impl Write>) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! serialize_primitive_le {
    ($T:ty) => {
        impl Serialize for $T {
            fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
                serializer.write_bytes(&self.to_le_bytes())?;
                Ok(())
            }
        }
    };
}

serialize_primitive_le!(u8);
serialize_primitive_le!(u16);
serialize_primitive_le!(u32);
serialize_primitive_le!(u64);

serialize_primitive_le!(i8);
serialize_primitive_le!(i16);
serialize_primitive_le!(i32);
serialize_primitive_le!(i64);

serialize_primitive_le!(f32);
serialize_primitive_le!(f64);

macro_rules! serialize_nonzero_primitive_le {
    ($NonZero:ty) => {
        impl Serialize for $NonZero {
            fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
                self.get().serialize(serializer)
            }
        }
    };
}

serialize_nonzero_primitive_le!(NonZeroU8);
serialize_nonzero_primitive_le!(NonZeroU16);
serialize_nonzero_primitive_le!(NonZeroU32);
serialize_nonzero_primitive_le!(NonZeroU64);

serialize_nonzero_primitive_le!(NonZeroI8);
serialize_nonzero_primitive_le!(NonZeroI16);
serialize_nonzero_primitive_le!(NonZeroI32);
serialize_nonzero_primitive_le!(NonZeroI64);

macro_rules! serialize_optional_nonzero_primitive_le {
    ($NonZero:ty) => {
        impl Serialize for Option<$NonZero> {
            fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
                self.clone()
                    .map(|x| x.get())
                    .unwrap_or(0)
                    .serialize(serializer)
            }
        }
    };
}

serialize_optional_nonzero_primitive_le!(NonZeroU8);
serialize_optional_nonzero_primitive_le!(NonZeroU16);
serialize_optional_nonzero_primitive_le!(NonZeroU32);
serialize_optional_nonzero_primitive_le!(NonZeroU64);

serialize_optional_nonzero_primitive_le!(NonZeroI8);
serialize_optional_nonzero_primitive_le!(NonZeroI16);
serialize_optional_nonzero_primitive_le!(NonZeroI32);
serialize_optional_nonzero_primitive_le!(NonZeroI64);

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
        // TODO: Bounds checking?
        (self.len() as u32)
            .serialize(serializer)
            .context("cannot serialize length of array")?;
        for (i, element) in self.iter().enumerate() {
            element
                .serialize(serializer)
                .with_context(|| format!("cannot serialize array element at index {i}"))?;
        }
        Ok(())
    }
}

impl Serialize for Uuid {
    fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
        serializer.write_bytes(&self.to_bytes_le())?;
        Ok(())
    }
}

pub fn serialize(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    let mut buffer = vec![];
    value.serialize(&mut Serializer::new(Cursor::new(&mut buffer)))?;
    Ok(buffer)
}
