//! Generation of `Script/Manifest.txt` files for UnrealEd.
//!
//! The `Manifest.txt` file format informs the editor about the entire available class hierarchy.
//! The details about why this file has to exist and why this data cannot just be derived at runtime
//! are probably lost to time, but UnrealEd needs this file to display actor classes correctly
//! (and will not even launch if it's not present).
//!
//! # Structure
//!
//! A `Manifest.txt` file begins with a single line containing the number `4`, presumably the
//! format version.
//!
//! ```text
//! 4
//! ```
//!
//! Each line afterwards contains the metadata for a single class, which looks like so:
//!
//! ```text
//! 0 Pckfdu Core [A] []
//!  1 Bdups Engine [A] []
//!   2 Csvti Engine [] []
//! ```
//!
//! Reading from the left, we have the inheritance depth, which is prepended with the same amount
//! of spaces, then separated by a space is the class name - which in case of A Hat in Time is
//! obfuscated by shifting all characters upwards by 1 in the ASCII table - followed by the module
//! name, which is oddly not obfuscated.
//!
//! The next part is class flags, which are enclosed in square brackets `[]`. The following flags
//! have been observed to be generated by Unreal, occurring in this exact order:
//!
//! - `P` - Placeable
//! - `H` - Deprecated (**H** for **h**idden?)
//! - `A` - Abstract
//! - `E` - Unknown
//!
//! Class flags can freely be combined, for example one can observe `[PA]` being present on certain
//! classes, which means that an actor is `placeable` but also `abstract`. Which might not make
//! sense initially, but consider that placeability is inherited from the parent class.
//!
//! At the end of the line we have a list of bracket-enclosed comma-separated class groups, which
//! allow the editor to group classes by... well, their group, at the user's request.

use thiserror::Error;

mod structure;
mod writer;

pub use structure::*;
pub use writer::*;

/// An error that can occur during generation.
#[derive(Debug, Error)]
pub enum Error {
    /// Formatting a value failed.
    #[error("formatting error")]
    Fmt(#[from] std::fmt::Error),

    /// Occurs when a class name contains characters that when shifted upward by 1 would result in
    /// invalid Unicode.
    #[error("when serializing the class name {0:?}, shifting the character {1} would result in invalid Unicode")]
    CharNotShiftable(String, char),
}
