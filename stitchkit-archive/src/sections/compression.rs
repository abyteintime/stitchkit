use anyhow::ensure;
use stitchkit_core::serializable_structure;

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
