use stitchkit_core::{string::UnrealString, uuid::Uuid, Deserialize};

use super::CompressedChunkPointer;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GenerationInfo {
    pub export_count: u32,
    pub name_count: u32,
    pub net_object_count: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
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
