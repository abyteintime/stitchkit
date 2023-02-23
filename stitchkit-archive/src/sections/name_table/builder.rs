use std::{collections::HashMap, ffi::CString};

use stitchkit_core::{flags::ObjectFlags, string::UnrealString};
use thiserror::Error;

use crate::name::ArchivedName;

use super::{NameTable, NameTableEntry};

#[derive(Debug, Clone, Default)]
pub struct NameTableBuilder {
    names: HashMap<String, ArchivedName>,
}

impl NameTableBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_or_insert(&mut self, name: &str) -> Result<ArchivedName, Error> {
        if let Some(&name) = self.names.get(name) {
            Ok(name)
        } else {
            // NOTE: We use i32::MAX here since Unreal uses a signed int internally, and who knows
            //  what might happen if we pass it a negative i32.
            //  My guesses are all bad things.
            if self.names.len() > i32::MAX as usize {
                return Err(Error::TooManyNames);
            }

            let index = self.names.len() as u32;
            let archived_name = ArchivedName {
                index,
                serial_number: 0,
            };
            self.names.insert(name.to_string(), archived_name);
            Ok(archived_name)
        }
    }

    pub fn build(self) -> Result<NameTable, Error> {
        let mut entries = vec![NameTableEntry::default(); self.names.len()];
        for (name_string, name) in self.names {
            let entry = &mut entries[name.index as usize];
            entry.name = UnrealString::from(
                CString::new(name_string.clone())
                    .map_err(|_| Error::NameHasNulBytes(name_string))?,
            );
            entry.flags = ObjectFlags::NAME;
        }
        Ok(NameTable { entries })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("name {0:?} contains nul bytes")]
    NameHasNulBytes(String),
    #[error("too many names (maximum of 2147483647 exceeded)")]
    TooManyNames,
}
