use std::{cmp::Ordering, fmt, io::Read, num::NonZeroU32};

use anyhow::Context;
use stitchkit_core::binary::{Deserialize, ReadExt};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PackageObjectIndex {
    Imported(NonZeroU32),
    Class,
    Exported(NonZeroU32),
}

impl fmt::Debug for PackageObjectIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Imported(i) => write!(f, "Imported({i})"),
            Self::Class => write!(f, "Class"),
            Self::Exported(i) => write!(f, "Exported({i})"),
        }
    }
}

impl Deserialize for PackageObjectIndex {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let index = reader
            .deserialize::<i32>()
            .context("cannot deserialize package object index")?;
        Ok(match index.cmp(&0) {
            Ordering::Less => Self::Imported(unsafe { NonZeroU32::new_unchecked(-index as u32) }),
            Ordering::Equal => Self::Class,
            Ordering::Greater => Self::Exported(unsafe { NonZeroU32::new_unchecked(index as u32) }),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OptionalPackageObjectIndex {
    Imported(NonZeroU32),
    None,
    Exported(NonZeroU32),
}

impl fmt::Debug for OptionalPackageObjectIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Imported(i) => write!(f, "Imported({i})"),
            Self::None => write!(f, "None"),
            Self::Exported(i) => write!(f, "Exported({i})"),
        }
    }
}

impl Deserialize for OptionalPackageObjectIndex {
    fn deserialize(mut reader: impl Read) -> anyhow::Result<Self> {
        let index = reader
            .deserialize::<i32>()
            .context("cannot deserialize package object index")?;
        Ok(match index.cmp(&0) {
            Ordering::Less => Self::Imported(unsafe { NonZeroU32::new_unchecked(-index as u32) }),
            Ordering::Equal => Self::None,
            Ordering::Greater => Self::Exported(unsafe { NonZeroU32::new_unchecked(index as u32) }),
        })
    }
}
