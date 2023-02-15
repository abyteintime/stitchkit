use std::{fmt, io::Read};

use anyhow::{ensure, Context};

use crate::binary::{Deserialize, Deserializer};

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
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let underlying = deserializer
            .deserialize::<u32>()
            .context("cannot deserialize Bool32")?;
        ensure!(
            underlying == 0 || underlying == 1,
            "Bool32 has invalid value (must be 0 or 1)"
        );
        Ok(Self(underlying))
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
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $NewType<const VALUE: $Underlying>;

        impl<const VALUE: $Underlying> Deserialize for $NewType<VALUE> {
            fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
                let value = deserializer.deserialize::<$Underlying>()?;
                ensure!(
                    value == VALUE,
                    "constant {} expected, but got {}",
                    VALUE,
                    value
                );
                Ok(Self)
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
