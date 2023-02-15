use std::{cmp::Ordering, fmt, io::Read, num::NonZeroU32, str::FromStr};

use anyhow::{bail, Context};
use stitchkit_core::binary::{Deserialize, Deserializer};

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

impl FromStr for PackageObjectIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "class" {
            Ok(Self::Class)
        } else if let Some(number) = s.strip_prefix("export:") {
            Ok(Self::Exported(number.parse()?))
        } else if let Some(number) = s.strip_prefix("import:") {
            Ok(Self::Imported(number.parse()?))
        } else {
            bail!("invalid package object index; it must be 'class', 'export:n', or 'import:n' where n is a 1-based index into the package's export/import table")
        }
    }
}

impl Deserialize for PackageObjectIndex {
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let index = deserializer
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
    fn deserialize(mut deserializer: Deserializer<impl Read>) -> anyhow::Result<Self> {
        let index = deserializer
            .deserialize::<i32>()
            .context("cannot deserialize package object index")?;
        Ok(match index.cmp(&0) {
            Ordering::Less => Self::Imported(unsafe { NonZeroU32::new_unchecked(-index as u32) }),
            Ordering::Equal => Self::None,
            Ordering::Greater => Self::Exported(unsafe { NonZeroU32::new_unchecked(index as u32) }),
        })
    }
}
