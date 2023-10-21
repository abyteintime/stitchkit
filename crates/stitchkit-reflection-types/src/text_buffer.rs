use stitchkit_archive::name::ArchivedName;
use stitchkit_core::{primitive::ConstU64, string::UnrealString, Deserialize, Serialize};

use crate::Object;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextBuffer {
    pub object: Object<ArchivedName>,
    pub _unknown: ConstU64<0>,
    pub text: UnrealString,
}
