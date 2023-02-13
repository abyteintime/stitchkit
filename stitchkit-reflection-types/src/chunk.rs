use stitchkit_archive::index::OptionalPackageObjectIndex;
use stitchkit_core::serializable_structure;

use crate::Field;

/// Equivalent of an Unreal `UStruct`.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub field: Field,
    /// If present, indicates the "parent chunk." This may mean different things depending on the
    /// type of chunk. For instance, in functions it's the function this chunk overrides, in classes
    /// this is the parent class, so on and so forth.
    pub parent_chunk: OptionalPackageObjectIndex,
    /// Always zero.
    pub zero_1: u32,
    /// The first parameter of this function.
    pub first_parameter: OptionalPackageObjectIndex,
    /// Always zero.
    pub zero_2: u32,
    /// Line number. This may be off by a few lines because `defaultproperties` blocks are
    /// incredibly janky.
    pub line_number: u32,
    /// The byte at which this chunk is declared in the file. This doesn't seem to be entirely
    /// accurate all the time, possibly due to line endings being converted to LF in the compiler?
    pub file_position: u32,
    /// The length of the declaration? This also seems to be hardly accurate.
    pub file_length: u32,
    /// The function's bytecode.
    pub bytecode: Vec<u8>,
}

serializable_structure! {
    type Chunk {
        field,
        parent_chunk,
        zero_1,
        first_parameter,
        zero_2,
        line_number,
        file_position,
        file_length,
        bytecode,
    }
}
