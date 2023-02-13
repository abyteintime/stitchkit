use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::ReadExt, flags::ObjectFlags, serializable_structure, uuid::Uuid};
use tracing::debug;

use crate::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    name::ArchivedName,
};

use super::Summary;

#[derive(Clone, Debug)]
pub struct ObjectExport {
    pub class_index: PackageObjectIndex,
    pub super_index: OptionalPackageObjectIndex,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
    pub archetype: OptionalPackageObjectIndex,
    pub object_flags: ObjectFlags,
    pub serial_size: u32,
    pub serial_offset: u32,
    pub export_flags: u32,
    pub net_object_count: u32,
    pub uuid: Uuid,
    pub unknown: u32,
}

serializable_structure! {
    type ObjectExport {
        class_index,
        super_index,
        outer_index,
        object_name,
        archetype,
        object_flags,
        serial_size,
        serial_offset,
        export_flags,
        net_object_count,
        uuid,
        unknown,
    }
}

impl Summary {
    pub fn deserialize_export_table(
        &self,
        mut reader: impl Read + Seek,
    ) -> anyhow::Result<Vec<ObjectExport>> {
        debug!(
            "Deserializing export table ({} exports at {:08x})",
            self.export_count, self.export_offset
        );
        reader.seek(SeekFrom::Start(self.export_offset as u64))?;
        let mut exports = Vec::with_capacity(self.export_count as usize);
        for i in 0..self.export_count {
            exports.push(
                reader
                    .deserialize()
                    .with_context(|| format!("cannot deserialize export {i}"))?,
            );
        }
        Ok(exports)
    }
}
