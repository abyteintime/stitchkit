use stitchkit_core::{binary::Deserialize, Deserialize};

/// Base object.
///
/// X can be used to stuff extra data after the index, in case the class in question does that.
#[derive(Debug, Clone, Deserialize)]
pub struct Object<X>
where
    X: Deserialize,
{
    /// -1 when the archive is uncooked.
    pub index_in_archive: i32,
    pub extra: X,
}
