use stitchkit_core::{
    primitive::{ConstU16, ConstU32},
    serializable_bitflags,
    string::UnrealString,
    uuid::Uuid,
    Deserialize, Serialize,
};

use crate::hat;

use super::CompressedChunkPointer;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GenerationInfo {
    pub export_count: u32,
    pub name_count: u32,
    pub net_object_count: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Summary {
    pub magic: ConstU32<{ hat::ARCHIVE_MAGIC }>,
    pub file_version: ConstU16<{ hat::ARCHIVE_FORMAT_VERSION }>,
    pub licensee_version: ConstU16<{ hat::ARCHIVE_LICENSEE_FORMAT_VERSION }>,
    pub headers_size: u32,
    pub package_group: UnrealString,
    pub package_flags: PackageFlags,

    pub name_table_len: u32,
    pub name_table_offset: u32,
    pub export_table_len: u32,
    pub export_table_offset: u32,
    pub import_table_len: u32,
    pub import_table_offset: u32,
    pub dependency_table_offset: u32,

    pub headers_size_mirror: u32,

    pub unknown_1: ConstU32<0>,
    pub unknown_2: ConstU32<0>,
    pub unknown_3: ConstU32<0>,

    pub uuid: Uuid,
    pub generations: Vec<GenerationInfo>,

    pub engine_version: ConstU32<{ hat::ENGINE_VERSION }>,
    pub cooker_version: u32,

    pub compression_kind: u32,
    pub compressed_chunks: Vec<CompressedChunkPointer>,

    // Unreal doesn't actually seem to care about this number.
    pub unknown_4: u32,
    pub unknown_5: ConstU32<0>,
    // This one seems to be important for textures, however I have not looked into it yet.
    // It's an array length, I think. Look at HatinTimeGame.u for an example.
    pub unknown_6: u32,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PackageFlags: u32 {
        // Meaning unknown, but they seem to be present on all packages.
        const COMMON = 0x00200001;
    }
}

serializable_bitflags!(PackageFlags);
