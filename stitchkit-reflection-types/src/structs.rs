#![allow(clippy::manual_strip)]

use bitflags::bitflags;
use stitchkit_core::{binary::TrailingData, serializable_bitflags, Deserialize};

use crate::Chunk;

#[derive(Debug, Clone, Deserialize)]
pub struct Struct {
    pub chunk: Chunk<()>,
    pub flags: StructFlags,
    pub trailing_data: TrailingData,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StructFlags: u32 {
    }
}

serializable_bitflags!(StructFlags);
