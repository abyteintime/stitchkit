use stitchkit_archive::index::OptionalPackageObjectIndex;
use stitchkit_core::{binary::Deserialize, Deserialize};

use crate::Object;

#[derive(Debug, Clone, Deserialize)]
pub struct Field<X = ()>
where
    X: Deserialize,
{
    pub object: Object<X>,
    pub next_object: OptionalPackageObjectIndex,
}
