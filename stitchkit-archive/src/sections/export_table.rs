pub mod unlinked;

use std::io::{Read, Seek, SeekFrom};

use anyhow::{anyhow, Context};
use stitchkit_core::{
    binary::{deserialize, Deserialize, Deserializer},
    flags::ObjectFlags,
    uuid::Uuid,
    Deserialize, Serialize,
};
use tracing::{debug, trace};

use crate::{
    index::{ExportIndex, OptionalPackageObjectIndex, PackageClassIndex},
    name::ArchivedName,
};

use super::Summary;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectExport {
    pub class_index: PackageClassIndex,
    pub super_index: OptionalPackageObjectIndex,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
    pub archetype: OptionalPackageObjectIndex,
    pub object_flags: ObjectFlags,
    pub serial_size: u32,
    pub serial_offset: u32,
    pub export_flags: u32,
    pub unknown_list: Vec<u32>,
    pub uuid: Uuid,
    pub unknown_flags: u32,
}

#[derive(Debug, Clone)]
pub struct ExportTable {
    pub exports: Vec<ObjectExport>,
}

impl ExportTable {
    pub fn get(&self, index: impl Into<ExportIndex>) -> Option<&ObjectExport> {
        self.exports.get(index.into().0 as usize)
    }

    pub fn try_get(
        &self,
        index: impl TryInto<ExportIndex, Error = anyhow::Error>,
    ) -> anyhow::Result<&ObjectExport> {
        let index = index.try_into()?;
        self.get(index)
            .ok_or_else(|| anyhow!("{index:?} is outside the bounds of the export table"))
    }

    pub fn push(&mut self, export: ObjectExport) -> ExportIndex {
        // TODO: Error handling here in case there are too many imports.
        let index = ExportIndex(self.exports.len() as u32);
        self.exports.push(export);
        index
    }
}

impl ObjectExport {
    pub fn get_serial_data<'a>(&self, archive: &'a [u8]) -> &'a [u8] {
        &archive
            [self.serial_offset as usize..self.serial_offset as usize + self.serial_size as usize]
    }

    pub fn deserialize_serial_data<T>(&self, archive: &[u8]) -> anyhow::Result<T>
    where
        T: Deserialize,
    {
        deserialize(self.get_serial_data(archive))
    }
}

impl Summary {
    pub fn deserialize_export_table(
        &self,
        deserializer: &mut Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<ExportTable> {
        debug!(
            "Deserializing export table ({} exports at {:08x})",
            self.export_table_len, self.export_table_offset
        );
        deserializer.seek(SeekFrom::Start(self.export_table_offset as u64))?;
        let mut exports = Vec::with_capacity(self.export_table_len as usize);
        for i in 0..self.export_table_len {
            trace!(
                "Export {} at position {:08x}",
                i + 1,
                deserializer.stream_position()
            );
            exports.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize export {i}"))?,
            );
        }
        Ok(ExportTable { exports })
    }
}
