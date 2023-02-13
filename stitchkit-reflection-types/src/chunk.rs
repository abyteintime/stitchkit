use stitchkit_archive::index::OptionalPackageObjectIndex;
use stitchkit_core::{binary::Deserialize, Deserialize};

use crate::Field;

/// Equivalent of an Unreal `UStruct`.
#[derive(Debug, Clone, Deserialize)]
pub struct Chunk<X = ()>
where
    X: Deserialize,
{
    pub field: Field<X>,
    /// If present, indicates the "parent chunk." This may mean different things depending on the
    /// type of chunk. For instance, in functions it's the function this chunk overrides, in classes
    /// this is the parent class, so on and so forth.
    pub parent_chunk: OptionalPackageObjectIndex,
    /// Always zero.
    pub zero_1: u32,
    /// The first variable in this function.
    pub first_variable: OptionalPackageObjectIndex,
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
