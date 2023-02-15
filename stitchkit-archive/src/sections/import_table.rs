use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::Deserializer, Deserialize};
use tracing::{debug, trace};

use crate::{index::OptionalPackageObjectIndex, name::ArchivedName};

use super::{NameTableEntry, Summary};

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectImport {
    pub class_package: ArchivedName,
    pub class_name: ArchivedName,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
}

impl ObjectImport {
    /// Resolves the object import to named parts `(package, class, object)`.
    pub fn resolve_names<'a>(
        &self,
        name_table: &'a [NameTableEntry],
    ) -> (&'a [u8], &'a [u8], &'a [u8]) {
        (
            name_table[self.class_package.index as usize]
                .name
                .to_bytes(),
            name_table[self.class_name.index as usize].name.to_bytes(),
            name_table[self.object_name.index as usize].name.to_bytes(),
        )
    }
}

impl Summary {
    pub fn deserialize_import_table(
        &self,
        mut deserializer: Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<Vec<ObjectImport>> {
        debug!(
            "Deserializing import table ({} imports at {:08x})",
            self.import_count, self.import_offset
        );
        deserializer.seek(SeekFrom::Start(self.import_offset as u64))?;
        let mut imports = Vec::with_capacity(self.import_count as usize);
        for i in 0..self.import_count {
            trace!(
                "Import {} at position {:08x}",
                i + 1,
                deserializer
                    .stream_position()
                    .context("cannot get stream position for trace logging")?
            );
            imports.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize import {i}"))?,
            );
        }
        Ok(imports)
    }
}
