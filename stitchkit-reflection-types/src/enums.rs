use stitchkit_archive::name::ArchivedName;
use stitchkit_core::{Deserialize, Serialize};

use crate::Field;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enum {
    pub field: Field<ArchivedName>,
    pub variants: Vec<ArchivedName>,
}
