use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::ReadExt, serializable_structure};
use tracing::debug;

use crate::{index::OptionalPackageObjectIndex, name::ArchivedName};

use super::Summary;

#[derive(Debug, Clone)]
pub struct ObjectImport {
    pub class_package: ArchivedName,
    pub class_name: ArchivedName,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
}

serializable_structure! {
    type ObjectImport {
        class_package,
        class_name,
        outer_index,
        object_name,
    }
}

impl Summary {
    pub fn deserialize_import_table(
        &self,
        mut reader: impl Read + Seek,
    ) -> anyhow::Result<Vec<ObjectImport>> {
        debug!(
            "Deserializing import table ({} imports at {:08x})",
            self.import_count, self.import_offset
        );
        reader.seek(SeekFrom::Start(self.import_offset as u64))?;
        let mut imports = Vec::with_capacity(self.import_count as usize);
        for i in 0..self.import_count {
            imports.push(
                reader
                    .deserialize()
                    .with_context(|| format!("cannot deserialize import {i}"))?,
            );
        }
        Ok(imports)
    }
}
