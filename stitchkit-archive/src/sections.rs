use std::io::{Read, Seek, SeekFrom};

use anyhow::{ensure, Context};
use stitchkit_core::{
    binary::ReadExt, flags::ObjectFlags, serializable_structure, string::UnrealString, uuid::Uuid,
};
use tracing::debug;

use crate::{
    hat,
    name::{ArchiveName, ArchiveNameDebug},
};

#[derive(Debug, Clone, Default)]
pub struct GenerationInfo {
    pub export_count: u32,
    pub name_count: u32,
    pub net_object_count: u32,
}

serializable_structure! {
    type GenerationInfo {
        export_count,
        name_count,
        net_object_count,
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompressedChunkPointer {
    pub uncompressed_offset: u32,
    pub uncompressed_size: u32,
    pub compressed_offset: u32,
    pub compressed_size: u32,
}

serializable_structure! {
    type CompressedChunkPointer {
        uncompressed_offset,
        uncompressed_size,
        compressed_offset,
        compressed_size,
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompressedChunkBlock {
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

serializable_structure! {
    type CompressedChunkBlock {
        compressed_size,
        uncompressed_size,
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompressedChunkHeader {
    pub magic: u32,
    pub block_size: u32,
    pub sum: CompressedChunkBlock,
}

serializable_structure! {
    type CompressedChunkHeader {
        magic,
        block_size,
        sum,
    }
    deserialize_extra |header| {
        ensure!(header.magic == 0x9E2A83C1, "mismatch in compressed chunk header tag");
    }
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub magic: u32,
    pub file_version: u16,
    pub licensee_version: u16,
    pub headers_size: u32,
    pub package_group: UnrealString,
    pub package_flags: u32,

    pub name_count: u32,
    pub name_offset: u32,
    pub export_count: u32,
    pub export_offset: u32,
    pub import_count: u32,
    pub import_offset: u32,
    pub depends_offset: u32,

    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,

    pub uuid: Uuid,
    pub generations: Vec<GenerationInfo>,

    pub engine_version: u32,
    pub cooker_version: u32,

    pub compression_kind: u32,
    pub compressed_chunks: Vec<CompressedChunkPointer>,
}

serializable_structure! {
    type Summary {
        magic,
        file_version,
        licensee_version,
        headers_size,
        package_group,
        package_flags,
        name_count,
        name_offset,
        export_count,
        export_offset,
        import_count,
        import_offset,
        depends_offset,
        unknown_1,
        unknown_2,
        unknown_3,
        unknown_4,
        uuid,
        generations,
        engine_version,
        cooker_version,
        compression_kind,
        compressed_chunks,
    }
    deserialize_extra |summary| {
        ensure!(
            summary.magic == hat::ARCHIVE_MAGIC,
            "archive magic number does not match {:08x} (it is {:08x})",
            hat::ARCHIVE_MAGIC,
            summary.magic
        );
    }
}

pub struct NameTableEntry {
    pub name: UnrealString,
    pub flags: u64,
}

serializable_structure! {
    type NameTableEntry {
        name,
        flags,
    }
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

#[derive(Clone)]
pub struct ObjectExport {
    /// Note that this, as well as other indices, is signed.
    /// - Positive values mean that the class resides within this archive.
    /// - Negative values indicate an import index.
    /// - 0 means `UClass`.
    pub class_index: i32,
    pub outer_index: i32,
    pub package_index: i32,
    pub object_name: ArchiveName,
    pub archetype: i32,
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
        outer_index,
        package_index,
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
            .field("super_index", &self.export.outer_index)
            .field("package_index", &self.export.package_index)
            .field(
                "object_name",
                &ArchiveNameDebug::new(self.name_table, self.export.object_name),
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

#[derive(Clone)]
pub struct ObjectImport {
    pub class_package: ArchiveName,
    pub class_name: ArchiveName,
    pub package_index: i32,
    pub object_name: ArchiveName,
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
                &ArchiveNameDebug::new(self.name_table, self.import.class_package),
            )
            .field(
                "class_name",
                &ArchiveNameDebug::new(self.name_table, self.import.class_name),
            )
            .field("package_index", &self.import.package_index)
            .field(
                "object_name",
                &ArchiveNameDebug::new(self.name_table, self.import.object_name),
            )
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectDependencies {
    pub dependencies: Vec<i32>,
}

serializable_structure! {
    type ObjectDependencies {
        dependencies,
    }
}

impl Summary {
    pub fn depends_count(&self) -> u32 {
        self.export_count
    }

    pub fn deserialize_dependency_table(
        &self,
        mut reader: impl Read + Seek,
    ) -> anyhow::Result<Vec<ObjectDependencies>> {
        debug!(
            "Deserializing dependency table ({} dependencies at {:08x})",
            self.depends_count(),
            self.depends_offset
        );
        reader.seek(SeekFrom::Start(self.depends_offset as u64))?;
        let mut depends = Vec::with_capacity(self.depends_count() as usize);
        for i in 0..self.depends_count() {
            depends.push(
                reader
                    .deserialize()
                    .with_context(|| format!("cannot deserialize dependency {i}"))?,
            );
        }
        Ok(depends)
    }
}
