//! High (and low) level structure of the `Manifest.txt` format.

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
    /// Flags that each class can be marked with.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ManifestFlags: u8 {
        /// `[P]` - the class is placeable.
        const PLACEABLE = 0x1;
        /// `[H]` - the class is deprecated (hidden?)
        const DEPRECATED = 0x2;
        /// `[A]` - the class is abstract (so non-placeable.)
        const ABSTRACT = 0x4;
        /// `[E]` - unknown.
        const UNKNOWN_E = 0x8;
    }
}

/// An owned `Manifest.txt` node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The class's name.
    pub class: String,
    /// The package this class belongs to.
    pub package: String,
    /// Flags informing the editor how to display the class.
    pub flags: ManifestFlags,
    /// `ClassGroup`s declared in the class.
    pub groups: Vec<String>,
    /// Classes that inherit from this one.
    pub children: Vec<Manifest>,
}

impl Manifest {
    /// Write the node to the given writer.
    pub fn write_to(&self, writer: &mut ManifestWriter<impl Write>) -> Result<(), Error> {
        writer.write_entry(Entry {
            class: &self.class,
            package: &self.package,
            flags: self.flags,
            groups: self.groups.iter().map(|x| x.deref()),
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
