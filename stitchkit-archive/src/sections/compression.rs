use stitchkit_core::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CompressedChunkPointer {
    pub uncompressed_offset: u32,
    pub uncompressed_size: u32,
    pub compressed_offset: u32,
    pub compressed_size: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CompressedChunkBlock {
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CompressedChunkHeader {
    pub magic: u32,
    pub block_size: u32,
    pub sum: CompressedChunkBlock,
}
