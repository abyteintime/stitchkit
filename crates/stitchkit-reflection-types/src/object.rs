use stitchkit_core::{
    binary::{Deserialize, Serialize},
    Deserialize, Serialize,
};

/// Base object.
///
/// X can be used to stuff extra data after the index, in case the class in question does that.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Object<X>
where
    X: Deserialize + Serialize,
{
    /// -1 when the archive is uncooked.
    pub index_in_archive: i32,
    pub extra: X,
}
