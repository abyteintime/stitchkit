use anyhow::ensure;
use stitchkit_core::{serializable_structure, string::UnrealString, uuid::Uuid};

use crate::hat;

use super::CompressedChunkPointer;

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
