use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::ReadExt, serializable_structure};
use tracing::debug;

use crate::name::{ArchivedName, ArchivedNameDebug};

use super::{NameTableEntry, Summary};

#[derive(Clone)]
pub struct ObjectImport {
    pub class_package: ArchivedName,
    pub class_name: ArchivedName,
    pub package_index: i32,
    pub object_name: ArchivedName,
}

serializable_structure! {
    type ObjectImport {
        class_package,
        class_name,
        package_index,
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

pub struct ObjectImportDebug<'a> {
    name_table: &'a [NameTableEntry],
    import: &'a ObjectImport,
}

impl<'a> ObjectImportDebug<'a> {
    pub fn new(name_table: &'a [NameTableEntry], import: &'a ObjectImport) -> Self {
        Self { name_table, import }
    }
}

impl<'a> std::fmt::Debug for ObjectImportDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectImport")
            .field(
                "class_package",
                &ArchivedNameDebug::new(self.name_table, self.import.class_package),
            )
            .field(
                "class_name",
                &ArchivedNameDebug::new(self.name_table, self.import.class_name),
            )
            .field("package_index", &self.import.package_index)
            .field(
                "object_name",
                &ArchivedNameDebug::new(self.name_table, self.import.object_name),
            )
            .finish()
    }
}
