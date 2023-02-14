use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName};
use stitchkit_core::Deserialize;

use crate::Chunk;

#[derive(Debug, Clone, Deserialize)]
pub struct State {
    pub chunk: Chunk<()>,
    pub unknown_1: u32,
    /// Always -1.
    pub unknown_2: i16,
    pub unknown_3: u32,
    pub function_map: Vec<FunctionMapEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionMapEntry {
    pub name: ArchivedName,
    pub function: OptionalPackageObjectIndex,
}
