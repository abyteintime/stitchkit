use std::{fmt, io::Read};

use anyhow::{ensure, Context};

use crate::binary::{Deserialize, ReadExt};

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
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let underlying = reader
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
