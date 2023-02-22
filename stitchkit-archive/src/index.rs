use std::{
    cmp::Ordering,
    fmt,
    io::{Read, Write},
    num::{NonZeroI32, NonZeroU32},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context};
use stitchkit_core::{
    binary::{Deserialize, Deserializer, Serialize, Serializer},
    Deserialize, Serialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportNumber(pub NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportNumber(pub NonZeroU32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportIndex(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportIndex(pub u32);

impl From<ExportIndex> for ExportNumber {
    fn from(value: ExportIndex) -> Self {
        // SAFETY: adding 1 to any u32 makes it non-zero since 0 + 1 is 1.
        unsafe { Self(NonZeroU32::new_unchecked(value.0 + 1)) }
    }
}

impl From<ImportIndex> for ImportNumber {
    fn from(value: ImportIndex) -> Self {
        // SAFETY: adding 1 to any u32 makes it non-zero since 0 + 1 is 1.
        unsafe { Self(NonZeroU32::new_unchecked(value.0 + 1)) }
    }
}

impl From<ExportNumber> for ExportIndex {
    fn from(value: ExportNumber) -> Self {
        Self(value.0.get() - 1)
    }
}

impl From<ImportNumber> for ImportIndex {
    fn from(value: ImportNumber) -> Self {
        Self(value.0.get() - 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PackageObjectIndex(NonZeroI32);

impl PackageObjectIndex {
    pub fn new(index: NonZeroI32) -> Self {
        Self(index)
    }

    pub fn is_exported(&self) -> bool {
        i32::from(self.0) > 0
    }

    pub fn is_imported(&self) -> bool {
        i32::from(self.0) < 0
    }

    pub fn export_number(&self) -> Option<ExportNumber> {
        self.is_exported()
            // SAFETY: is_exported guarantees that self.0 is positive and non-zero, so
            // it's safe to create a NonZeroU32 from it.
            .then_some(unsafe { ExportNumber(NonZeroU32::new_unchecked(i32::from(self.0) as u32)) })
    }

    pub fn export_index(&self) -> Option<ExportIndex> {
        self.export_number().map(|x| x.into())
    }

    pub fn import_number(&self) -> Option<ImportNumber> {
        self.is_imported()
            // SAFETY: is_imported guarantees that self.0 is negative and non-zero, so by negating
            // it it's safe to create a NonZeroU32.
            .then_some(unsafe {
                ImportNumber(NonZeroU32::new_unchecked((-i32::from(self.0)) as u32))
            })
    }

    pub fn import_index(&self) -> Option<ImportIndex> {
        self.import_number().map(|x| x.into())
    }
}

impl TryFrom<PackageObjectIndex> for ExportIndex {
    type Error = anyhow::Error;

    fn try_from(value: PackageObjectIndex) -> Result<Self, Self::Error> {
        value
            .export_index()
            .ok_or_else(|| anyhow!("package object index is not an export"))
    }
}

impl From<ExportNumber> for PackageObjectIndex {
    fn from(value: ExportNumber) -> Self {
        // SAFETY: An ExportNumber is always non-zero.
        Self(unsafe { NonZeroI32::new_unchecked(value.0.get() as i32) })
    }
}

impl From<ExportIndex> for PackageObjectIndex {
    fn from(value: ExportIndex) -> Self {
        Self::from(ExportNumber::from(value))
    }
}

impl TryFrom<PackageObjectIndex> for ImportIndex {
    type Error = anyhow::Error;

    fn try_from(value: PackageObjectIndex) -> Result<Self, Self::Error> {
        value
            .import_index()
            .ok_or_else(|| anyhow!("package object index is not an import"))
    }
}

impl From<ImportNumber> for PackageObjectIndex {
    fn from(value: ImportNumber) -> Self {
        // SAFETY: An ImportNumber is always non-zero.
        Self(unsafe { NonZeroI32::new_unchecked(-(value.0.get() as i32)) })
    }
}

impl From<ImportIndex> for PackageObjectIndex {
    fn from(value: ImportIndex) -> Self {
        Self::from(ImportNumber::from(value))
    }
}

impl From<PackageObjectIndex> for NonZeroI32 {
    fn from(value: PackageObjectIndex) -> Self {
        value.0
    }
}

impl From<PackageObjectIndex> for i32 {
    fn from(value: PackageObjectIndex) -> Self {
        value.0.get()
    }
}

impl fmt::Debug for PackageObjectIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ExportNumber(export)) = self.export_number() {
            write!(f, "Exported({export})")
        } else if let Some(ImportNumber(import)) = self.import_number() {
            write!(f, "Imported({import})")
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OptionalPackageObjectIndex(pub Option<PackageObjectIndex>);

impl OptionalPackageObjectIndex {
    pub fn new(index: i32) -> Self {
        Self(NonZeroI32::new(index).map(PackageObjectIndex::new))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_exported(&self) -> bool {
        self.0.map(|x| x.is_exported()).unwrap_or(false)
    }

    pub fn is_imported(&self) -> bool {
        self.0.map(|x| x.is_imported()).unwrap_or(false)
    }

    pub fn export_number(&self) -> Option<ExportNumber> {
        self.0.and_then(|x| x.export_number())
    }

    pub fn export_index(&self) -> Option<ExportIndex> {
        self.0.and_then(|x| x.export_index())
    }

    pub fn import_number(&self) -> Option<ImportNumber> {
        self.0.and_then(|x| x.import_number())
    }

    pub fn import_index(&self) -> Option<ImportIndex> {
        self.0.and_then(|x| x.import_index())
    }
}

impl From<Option<PackageObjectIndex>> for OptionalPackageObjectIndex {
    fn from(value: Option<PackageObjectIndex>) -> Self {
        Self(value)
    }
}

impl From<OptionalPackageObjectIndex> for Option<PackageObjectIndex> {
    fn from(value: OptionalPackageObjectIndex) -> Self {
        value.0
    }
}

impl From<ExportIndex> for OptionalPackageObjectIndex {
    fn from(value: ExportIndex) -> Self {
        Self(Some(PackageObjectIndex::from(value)))
    }
}

impl From<ExportNumber> for OptionalPackageObjectIndex {
    fn from(value: ExportNumber) -> Self {
        Self(Some(PackageObjectIndex::from(value)))
    }
}

impl From<ImportIndex> for OptionalPackageObjectIndex {
    fn from(value: ImportIndex) -> Self {
        Self(Some(PackageObjectIndex::from(value)))
    }
}

impl From<ImportNumber> for OptionalPackageObjectIndex {
    fn from(value: ImportNumber) -> Self {
        Self(Some(PackageObjectIndex::from(value)))
    }
}

impl From<OptionalPackageObjectIndex> for i32 {
    fn from(value: OptionalPackageObjectIndex) -> Self {
        value.0.map(i32::from).unwrap_or(0)
    }
}

impl TryFrom<OptionalPackageObjectIndex> for ExportIndex {
    type Error = anyhow::Error;

    fn try_from(value: OptionalPackageObjectIndex) -> Result<Self, Self::Error> {
        value
            .export_index()
            .ok_or_else(|| anyhow!("package object index is not an export"))
    }
}

impl TryFrom<OptionalPackageObjectIndex> for ImportIndex {
    type Error = anyhow::Error;

    fn try_from(value: OptionalPackageObjectIndex) -> Result<Self, Self::Error> {
        value
            .import_index()
            .ok_or_else(|| anyhow!("package object index is not an import"))
    }
}

impl fmt::Debug for OptionalPackageObjectIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Deserialize for OptionalPackageObjectIndex {
    fn deserialize(deserializer: &mut Deserializer<impl Read>) -> anyhow::Result<Self> {
        let index = deserializer
            .deserialize::<i32>()
            .context("cannot deserialize OptionalPackageObjectIndex")?;
        Ok(match index.cmp(&0) {
            // SAFETY: Since index is <> 0, that means it's safe to create a NonZeroI32 from it.
            Ordering::Less | Ordering::Greater => Self(Some(PackageObjectIndex(unsafe {
                NonZeroI32::new_unchecked(index)
            }))),
            Ordering::Equal => Self(None),
        })
    }
}

impl Serialize for OptionalPackageObjectIndex {
    fn serialize(&self, serializer: &mut Serializer<impl Write>) -> anyhow::Result<()> {
        i32::from(*self).serialize(serializer)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageClassIndex {
    index: OptionalPackageObjectIndex,
}

impl PackageClassIndex {
    pub fn new(index: i32) -> Self {
        Self {
            index: OptionalPackageObjectIndex::new(index),
        }
    }

    pub fn class() -> Self {
        Self {
            index: OptionalPackageObjectIndex(None),
        }
    }

    pub fn is_class(&self) -> bool {
        self.index.is_none()
    }

    pub fn is_exported(&self) -> bool {
        self.index.is_exported()
    }

    pub fn is_imported(&self) -> bool {
        self.index.is_imported()
    }

    pub fn export_number(&self) -> Option<ExportNumber> {
        self.index.export_number()
    }

    pub fn export_index(&self) -> Option<ExportIndex> {
        self.index.export_index()
    }

    pub fn import_number(&self) -> Option<ImportNumber> {
        self.index.import_number()
    }

    pub fn import_index(&self) -> Option<ImportIndex> {
        self.index.import_index()
    }
}

impl From<PackageClassIndex> for OptionalPackageObjectIndex {
    fn from(value: PackageClassIndex) -> Self {
        value.index
    }
}

impl fmt::Debug for PackageClassIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index.0 {
            Some(index) => fmt::Debug::fmt(&index, f),
            None => f.write_str("Class"),
        }
    }
}

impl FromStr for PackageClassIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            index: if s == "class" {
                OptionalPackageObjectIndex(None)
            } else if let Some(number) = s.strip_prefix("export:") {
                let index = number.parse::<u32>()?;
                OptionalPackageObjectIndex(Some(PackageObjectIndex(
                    NonZeroI32::new(i32::try_from(index)?)
                        .ok_or_else(|| anyhow!("class index must not be zero"))?,
                )))
            } else if let Some(number) = s.strip_prefix("import:") {
                let index = number.parse::<u32>()?;
                OptionalPackageObjectIndex(Some(PackageObjectIndex(
                    NonZeroI32::new(-i32::try_from(index)?)
                        .ok_or_else(|| anyhow!("class index must not be zero"))?,
                )))
            } else {
                bail!("invalid package object index; it must be 'class', 'export:n', or 'import:n' where n is a 1-based index into the package's export/import table")
            },
        })
    }
}
