use std::{
    fmt::{Display, Formatter, Write},
    ops::Deref,
};

use bitflags::bitflags;

use crate::{
    writer::{Entry, ManifestWriter},
    Error,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ManifestFlags: u8 {
        const PLACEABLE = 0x1;     // P
        const DEPRECATED = 0x2;    // H
        const ABSTRACT = 0x4;      // A
        const UNKNOWN_E = 0x8;     // E
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub class: String,
    pub package: String,
    pub flags: ManifestFlags,
    pub categories: Vec<String>,
    pub children: Vec<Manifest>,
}

impl Manifest {
    pub fn write_to(&self, writer: &mut ManifestWriter<impl Write>) -> Result<(), Error> {
        writer.write_entry(Entry {
            class: &self.class,
            package: &self.package,
            flags: self.flags,
            groups: self.categories.iter().map(|x| x.deref()),
        })?;
        writer.descend();
        for child in &self.children {
            child.write_to(writer)?;
        }
        writer.ascend();
        Ok(())
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = ManifestWriter::new(f).map_err(|_| std::fmt::Error)?;
        self.write_to(&mut writer).map_err(|_| std::fmt::Error)
    }
}
