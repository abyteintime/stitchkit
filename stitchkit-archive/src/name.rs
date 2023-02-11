use crate::{sections::NameTableEntry, serializable_structure};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Name {
    index: u32,
    serial_number: u32,
}

serializable_structure! {
    type Name {
        index,
        serial_number,
    }
}

pub struct NameDebug<'a> {
    name_table: &'a [NameTableEntry],
    name: Name,
}

impl<'a> NameDebug<'a> {
    pub fn new(name_table: &'a [NameTableEntry], name: Name) -> Self {
        Self { name_table, name }
    }
}

impl<'a> std::fmt::Debug for NameDebug<'a> {
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
