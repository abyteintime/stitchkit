use std::fmt::Write;

use crate::structure::ManifestFlags;
use crate::Error;

/// A `Manifest.txt` writer.
#[derive(Debug, Clone)]
pub struct ManifestWriter<W> {
    level: usize,
    writer: W,
}

/// A single, non-recursive entry inside the `Manifest.txt` file, describing a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a, I> {
    /// The class's name.
    pub class: &'a str,
    /// The package the class belongs to.
    pub package: &'a str,
    /// Flags informing the editor how to display the class.
    pub flags: ManifestFlags,
    /// An iterator over this class's declared `ClassGroup`s.
    pub groups: I,
}

impl<W> ManifestWriter<W>
where
    W: Write,
{
    /// Create a new manifest writer, writing to the provided output stream.
    pub fn new(mut writer: W) -> Result<Self, Error> {
        // Version I suppose?
        writer.write_str("4\n")?;
        Ok(Self { level: 0, writer })
    }

    /// Descend by one inheritance level. This must be balanced out with an [`ascend`][Self::ascend]
    /// call afterwards.
    ///
    /// # Example
    /// ```
    /// let mut manifest = String::new();
    /// let mut writer = ManifestWriter::new(&mut manifest).unwrap();
    /// writer.write_entry(Entry {
    ///     class: "Object",
    ///     package: "Core",
    ///     flags: ManifestFlags::ABSTRACT,
    ///     groups: std::iter::empty(),
    /// });
    /// writer.descend();
    /// writer.write_entry(Entry {
    ///     class: "Actor",
    ///     package: "Engine",
    ///     flags: ManifestFlags::default(),
    ///     groups: std::iter::empty(),
    /// });
    /// writer.ascend();
    /// ```
    pub fn descend(&mut self) {
        self.level += 1;
    }

    /// Ascend by an inheritance level. This must be balanced out with a [`descend`][Self::descend]
    /// call beforehand. See that function for examples.
    pub fn ascend(&mut self) {
        self.level -= 1;
    }

    /// Write a single entry to the manifest file.
    pub fn write_entry<'a>(
        &mut self,
        entry: Entry<'_, impl Iterator<Item = &'a str>>,
    ) -> Result<(), Error> {
        for _ in 0..self.level {
            self.writer.write_char(' ')?;
        }
        write!(self.writer, "{} ", self.level)?;

        for c in entry.class.chars() {
            // This is incredibly funny to me. ROT-1, I suppose.
            let cc = char::try_from(c as u32 + 1)
                .map_err(|_| Error::CharNotShiftable(entry.class.to_owned(), c))?;
            self.writer.write_char(cc)?;
        }
        write!(self.writer, " {} [", entry.package)?;

        if entry.flags.contains(ManifestFlags::PLACEABLE) {
            self.writer.write_char('P')?;
        }
        if entry.flags.contains(ManifestFlags::DEPRECATED) {
            self.writer.write_char('H')?;
        }
        if entry.flags.contains(ManifestFlags::ABSTRACT) {
            self.writer.write_char('A')?;
        }
        if entry.flags.contains(ManifestFlags::UNKNOWN_E) {
            self.writer.write_char('E')?;
        }

        self.writer.write_str("] [")?;
        for (i, category) in entry.groups.enumerate() {
            if i != 0 {
                self.writer.write_char(',')?;
            }
            self.writer.write_str(category)?;
        }
        self.writer.write_str("]\r\n")?;

        Ok(())
    }
}
