use stitchkit_core::{binary::Deserialize, Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Object<X = ()>
where
    X: Deserialize,
{
    /// -1 when the archive is uncooked.
    pub index_in_archive: i32,
    pub extra: X,
}
