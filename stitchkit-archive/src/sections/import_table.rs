use std::io::{Read, Seek, SeekFrom};

use stitchkit_core::{
    binary::{self, Deserializer, ResultContextExt},
    Deserialize, Serialize,
};
use tracing::{debug, trace};

use crate::{
    index::{ImportIndex, OptionalPackageObjectIndex},
    name::ArchivedName,
};

use super::{NameTable, Summary};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectImport {
    pub class_package: ArchivedName,
    pub class_name: ArchivedName,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
}

#[derive(Debug, Clone, Default)]
pub struct ImportTable {
    pub imports: Vec<ObjectImport>,
}

impl ImportTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, index: impl Into<ImportIndex>) -> Option<&ObjectImport> {
        self.imports.get(index.into().0 as usize)
    }

    pub fn push(&mut self, import: ObjectImport) -> ImportIndex {
        // TODO: Error handling here in case there are too many imports.
        let index = ImportIndex(self.imports.len() as u32);
        self.imports.push(import);
        index
    }
}

impl ObjectImport {
    /// Resolves the object import to named parts `(package, class, object)`.
    pub fn resolve_names<'a>(&self, name_table: &'a NameTable) -> (&'a [u8], &'a [u8], &'a [u8]) {
        (
            name_table
                .get_str(self.class_package.index as usize)
                .unwrap_or(b""),
            name_table
                .get_str(self.class_name.index as usize)
                .unwrap_or(b""),
            name_table
                .get_str(self.object_name.index as usize)
                .unwrap_or(b""),
        )
    }
}

impl Summary {
    pub fn deserialize_import_table(
        &self,
        deserializer: &mut Deserializer<impl Read + Seek>,
    ) -> Result<ImportTable, binary::Error> {
        debug!(
            "Deserializing import table ({} imports at {:08x})",
            self.import_table_len, self.import_table_offset
        );
        deserializer.seek(SeekFrom::Start(self.import_table_offset as u64))?;
        let mut imports = Vec::with_capacity(self.import_table_len as usize);
        for i in 0..self.import_table_len {
            trace!(
                "Import {} at position {:08x}",
                i + 1,
                deserializer.stream_position()
            );
            imports.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize import {i}"))?,
            );
        }
        Ok(ImportTable { imports })
    }
}
