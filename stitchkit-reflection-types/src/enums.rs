use stitchkit_archive::name::ArchivedName;
use stitchkit_core::Deserialize;

use crate::Field;

#[derive(Debug, Clone, Deserialize)]
pub struct Enum {
    pub field: Field<ArchivedName>,
    pub variants: Vec<ArchivedName>,
}
