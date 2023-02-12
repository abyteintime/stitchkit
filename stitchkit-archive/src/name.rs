use stitchkit_core::serializable_structure;

use crate::sections::NameTableEntry;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchiveName {
    index: u32,
    serial_number: u32,
}

serializable_structure! {
    type ArchiveName {
        index,
        serial_number,
    }
}

pub struct ArchiveNameDebug<'a> {
    name_table: &'a [NameTableEntry],
    name: ArchiveName,
}

impl<'a> ArchiveNameDebug<'a> {
    pub fn new(name_table: &'a [NameTableEntry], name: ArchiveName) -> Self {
        Self { name_table, name }
    }
}

impl<'a> std::fmt::Debug for ArchiveNameDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(entry) = self.name_table.get(self.name.index as usize) {
            f.write_str("'")?;
            std::fmt::Display::fmt(&entry.name, f)?;
            f.write_str("'")?;
        } else {
            write!(f, "<invalid name {}>", self.name.index)?;
        }
        if self.name.serial_number != 0 {
            write!(f, "_{}", self.name.serial_number)?;
        }
        Ok(())
    }
}
