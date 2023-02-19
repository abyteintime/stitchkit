use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::Deserializer, flags::ObjectFlags, string::UnrealString, Deserialize};
use tracing::{debug, trace};

use crate::name::ArchivedName;

use super::Summary;

#[derive(Debug, Clone, Deserialize)]
pub struct NameTableEntry {
    pub name: UnrealString,
    pub flags: ObjectFlags,
}

#[derive(Debug, Clone)]
pub struct NameTable {
    pub entries: Vec<NameTableEntry>,
}

impl NameTable {
    pub fn get(&self, index: usize) -> Option<&NameTableEntry> {
        self.entries.get(index)
    }

    pub fn get_str(&self, index: usize) -> Option<&[u8]> {
        self.get(index).map(|entry| entry.name.to_bytes())
    }

    pub fn name_to_str(&self, name: ArchivedName) -> Option<&[u8]> {
        self.get_str(name.index as usize)
    }
}

impl Summary {
    pub fn deserialize_name_table(
        &self,
        deserializer: &mut Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<NameTable> {
        debug!(
            "Deserializing name table ({} names at {:08x})",
            self.name_count, self.name_offset
        );
        deserializer.seek(SeekFrom::Start(self.name_offset as u64))?;
        let mut entries = Vec::with_capacity(self.name_count as usize);
        for i in 0..self.name_count {
            trace!(
                "Name {i} at position {:08x}",
                deserializer.stream_position()
            );
            entries.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize name {i}"))?,
            );
        }
        Ok(NameTable { entries })
    }
}
