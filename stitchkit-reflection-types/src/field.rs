use stitchkit_archive::index::PackageObjectIndex;
use stitchkit_core::serializable_structure;

use crate::Object;

#[derive(Debug, Clone)]
pub struct Field {
    pub object: Object,
    pub next_object: PackageObjectIndex,
}

serializable_structure! {
    type Field {
        object,
        next_object,
    }
}
