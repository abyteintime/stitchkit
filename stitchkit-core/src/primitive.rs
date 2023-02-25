use std::{
    fmt,
    io::{Read, Write},
};

use crate::binary::{
    Deserialize, Deserializer, Error, ErrorKind, ResultContextExt, Serialize, Serializer,
};

/// 32-bit Unreal `UBOOL`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bool32(u32);

impl From<bool> for Bool32 {
    fn from(value: bool) -> Self {
        Self(value as u32)
    }
}

impl From<Bool32> for bool {
    fn from(value: Bool32) -> Self {
        value.0 != 0
    }
}

impl Deserialize for Bool32 {
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
        let underlying = deserializer
            .deserialize::<u32>()
            .context("cannot deserialize Bool32")?;
        if underlying != 0 && underlying != 1 {
            Err(ErrorKind::Deserialize.make("Bool32 has invalid value (must be 0 or 1)"))
        } else {
            Ok(Self(underlying))
        }
    }
}

impl Serialize for Bool32 {
    fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
        self.0.serialize(serializer)
    }
}

impl fmt::Debug for Bool32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&bool::from(*self), f)
    }
}

impl fmt::Display for Bool32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&bool::from(*self), f)
    }
}

macro_rules! const_primitive {
    ($Underlying:ty, $NewType:tt) => {
        #[doc = concat!("Always serializes to the same `", stringify!($Underlying), "`.\n\nDuring deserialization if the value is not that constant, an error is thrown.")]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
        pub struct $NewType<const VALUE: $Underlying>;

        impl<const VALUE: $Underlying> Deserialize for $NewType<VALUE> {
            fn deserialize(deserializer: &mut Deserializer<impl Read>) -> Result<Self, Error> {
                let value = deserializer.deserialize::<$Underlying>()?;
                if value != VALUE {
                    Err(ErrorKind::Deserialize.make(format!("constant {VALUE} expected, but got {value}")))
                } else {
                    Ok(Self)
                }
            }
        }

        impl<const VALUE: $Underlying> Serialize for $NewType<VALUE> {
            fn serialize(&self, serializer: &mut Serializer<impl Write>) -> Result<(), Error> {
                VALUE.serialize(serializer)
            }
        }

        impl<const VALUE: $Underlying> fmt::Debug for $NewType<VALUE> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&VALUE, f)
            }
        }

        impl<const VALUE: $Underlying> fmt::Display for $NewType<VALUE> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&VALUE, f)
            }
        }
    }
}

const_primitive!(u8, ConstU8);
const_primitive!(u16, ConstU16);
const_primitive!(u32, ConstU32);
const_primitive!(u64, ConstU64);

const_primitive!(i8, ConstI8);
const_primitive!(i16, ConstI16);
const_primitive!(i32, ConstI32);
const_primitive!(i64, ConstI64);
