use stitchkit_core::{binary::TrailingData, Deserialize};

use crate::State;

#[derive(Debug, Clone, Deserialize)]
pub struct Class {
    pub state: State,
    pub trailing_data: TrailingData,
}
