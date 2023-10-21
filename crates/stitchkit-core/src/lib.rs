//! Stitchkit core library. Analogous to Unreal's `Core` module, it contains core types for
//! interfacing with Unreal.

pub mod binary;
pub mod context;
pub mod flags;
pub mod primitive;
pub mod string;

pub extern crate uuid;

pub use stitchkit_core_derive::*;
