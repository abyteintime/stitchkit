use stitchkit_archive::index::OptionalPackageObjectIndex;
use stitchkit_core::{
    binary::{Deserialize, Serialize},
    primitive::ConstU32,
    Deserialize, Serialize,
};

use crate::Field;

/// Equivalent of an Unreal `UStruct`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chunk<X>
where
    X: Deserialize + Serialize,
{
    pub field: Field<X>,
    /// If present, indicates the "parent chunk." This may mean different things depending on the
    /// type of chunk. For instance, in functions it's the function this chunk overrides, in classes
    /// this is the parent class, so on and so forth.
    pub parent_chunk: OptionalPackageObjectIndex,
    /// Classes use this field to store a pointer to a ScriptText object containing the source code.
    pub source_code: OptionalPackageObjectIndex,
    /// The first variable in this function or class.
    pub first_variable: OptionalPackageObjectIndex,
    /// Always zero.
    pub _zero: ConstU32<0>,
    /// Line number. This may be off by a few lines because `defaultproperties` blocks are
    /// incredibly janky.
    ///
    /// In classes this is -1.
    pub line_number: i32,
    /// The byte at which this chunk is declared in the file. This doesn't seem to be entirely
    /// accurate all the time, possibly due to line endings being converted to LF in the compiler?
    ///
    /// In classes this is -1.
    pub file_position: i32,
    /// The length of the declaration? This also seems to be hardly accurate.
    pub file_length: u32,
    /// The function's bytecode.
    pub bytecode: Vec<u8>,
}
