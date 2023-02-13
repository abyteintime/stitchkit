use stitchkit_archive::name::ArchivedName;
use stitchkit_core::serializable_structure;

#[derive(Debug, Clone)]
pub struct Object {
    /// -1 when the archive is uncooked.
    pub archive_index: i32,
    pub unknown_name: ArchivedName,
}

serializable_structure! {
    type Object {
        archive_index,
        unknown_name,
    }
}
