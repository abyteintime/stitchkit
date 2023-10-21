use std::fmt;

use stitchkit_core::{context, Deserialize, Serialize};

use crate::sections::NameTable;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ArchivedName {
    pub index: u32,
    pub serial_number: u32,
}

impl ArchivedName {
    pub fn is_none(&self, name_table: &NameTable) -> bool {
        name_table.name_to_str(*self) == Some(b"None")
    }
}

context! {
    pub let archived_name_table: NameTable;
}

impl fmt::Debug for ArchivedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name_table) = archived_name_table::get() {
            if let Some(entry) = name_table.get(self.index as usize) {
                f.write_str("'")?;
                fmt::Display::fmt(&entry.name, f)?;
                f.write_str("'")?;
            } else {
                write!(f, "<invalid name {}>", self.index)?;
            }
        } else {
            write!(f, "{}", self.index)?;
        }
        if self.serial_number != 0 {
            write!(f, "_{}", self.serial_number - 1)?;
        }
        Ok(())
    }
}
