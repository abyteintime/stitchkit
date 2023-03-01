//! Support for flexible binary serialization/deserialization.

#[doc(hidden)]
pub mod macros;

mod deserialization;
mod error;
mod serialization;
mod trailing_data;

pub use deserialization::*;
pub use error::*;
pub use serialization::*;
pub use trailing_data::*;
