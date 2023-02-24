use std::fmt::Write;

use crate::structure::ManifestFlags;
use crate::Error;

#[derive(Debug, Clone)]
pub struct ManifestWriter<W> {
    level: usize,
    writer: W,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a, I> {
    pub class: &'a str,
    pub package: &'a str,
    pub flags: ManifestFlags,
    pub groups: I,
}

impl<W> ManifestWriter<W>
where
    W: Write,
{
    pub fn new(mut writer: W) -> Result<Self, Error> {
        // Version I suppose?
        writer.write_str("4\n")?;
        Ok(Self { level: 0, writer })
    }

    pub fn descend(&mut self) {
        self.level += 1;
    }

    pub fn ascend(&mut self) {
        self.level -= 1;
    }

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
