pub mod walk;

use stitchkit_archive::index::OptionalPackageObjectIndex;
use stitchkit_core::{
    binary::{Deserialize, Serialize},
    Deserialize, Serialize,
};

use crate::Object;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Field<X>
where
    X: Deserialize + Serialize,
{
    pub object: Object<X>,
    pub next_object: OptionalPackageObjectIndex,
}
