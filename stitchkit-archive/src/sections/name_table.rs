use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::ReadExt, flags::ObjectFlags, string::UnrealString, Deserialize};
use tracing::debug;

use super::Summary;

#[derive(Debug, Clone, Deserialize)]
pub struct NameTableEntry {
    pub name: UnrealString,
    pub flags: ObjectFlags,
}

impl Summary {
    pub fn deserialize_name_table(
        &self,
        mut reader: impl Read + Seek,
    ) -> anyhow::Result<Vec<NameTableEntry>> {
        debug!(
            "Deserializing name table ({} names at {:08x})",
            self.name_count, self.name_offset
        );
        reader.seek(SeekFrom::Start(self.name_offset as u64))?;
        let mut names = Vec::with_capacity(self.name_count as usize);
        for i in 0..self.name_count {
            names.push(
                reader
                    .deserialize()
                    .with_context(|| format!("cannot deserialize name {i}"))?,
            );
        }
        Ok(names)
    }
}
