use std::io::{Read, Seek, SeekFrom};

use anyhow::{ensure, Context};
use tracing::debug;
use uuid::Uuid;

use crate::{binary::ReadExt, serializable_structure, string::UnrealString};

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
pub struct CompressedChunk {
    pub uncompressed_offset: u32,
    pub uncompressed_size: u32,
    pub compressed_offset: u32,
    pub compressed_size: u32,
}

serializable_structure! {
    type CompressedChunk {
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
    pub tag: u32,
    pub block_size: u32,
    pub sum: CompressedChunkBlock,
}

serializable_structure! {
    type CompressedChunkHeader {
        tag,
        block_size,
        sum,
    }
    deserialize_extra |header| {
        ensure!(header.tag == 0x9E2A83C1, "mismatch in compressed chunk header tag");
    }
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub tag: u32,
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
    pub compressed_chunks: Vec<CompressedChunk>,
}

serializable_structure! {
    type Summary {
        tag,
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

pub struct NameTable {
    pub names: Vec<NameTableEntry>,
}

impl Summary {
    pub fn deserialize_name_table(
        &self,
        mut reader: impl Read + Seek,
    ) -> anyhow::Result<NameTable> {
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
        Ok(NameTable { names })
    }
}
