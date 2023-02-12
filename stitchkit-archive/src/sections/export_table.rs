use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::ReadExt, flags::ObjectFlags, serializable_structure, uuid::Uuid};
use tracing::debug;

use crate::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    name::{ArchivedName, ArchivedNameDebug},
};

use super::{NameTableEntry, Summary};

#[derive(Clone)]
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

pub struct ObjectExportDebug<'a> {
    name_table: &'a [NameTableEntry],
    export: &'a ObjectExport,
}

impl<'a> ObjectExportDebug<'a> {
    pub fn new(name_table: &'a [NameTableEntry], export: &'a ObjectExport) -> Self {
        Self { name_table, export }
    }
}

impl<'a> std::fmt::Debug for ObjectExportDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectExport")
            .field("class_index", &self.export.class_index)
            .field("super_index", &self.export.super_index)
            .field("outer_index", &self.export.outer_index)
            .field(
                "object_name",
                &ArchivedNameDebug::new(self.name_table, self.export.object_name),
            )
            .field("archetype", &self.export.archetype)
            .field("object_flags", &self.export.object_flags)
            .field("serial_size", &self.export.serial_size)
            .field("serial_offset", &self.export.serial_offset)
            .field("export_flags", &self.export.export_flags)
            .field("net_object_count", &self.export.net_object_count)
            .field("uuid", &self.export.uuid)
            .field("unknown", &self.export.unknown)
            .finish()
    }
}
