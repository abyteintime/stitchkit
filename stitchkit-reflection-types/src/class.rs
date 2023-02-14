#![allow(clippy::manual_strip)]

use bitflags::bitflags;
use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    name::ArchivedName,
};
use stitchkit_core::{
    primitive::{Bool32, ConstU32},
    serializable_bitflags,
    string::UnrealString,
    Deserialize,
};
use tracing::warn;

use crate::State;

#[derive(Debug, Clone, Deserialize)]
pub struct Class {
    pub state: State,
    /// Flags that tell you information about the class.
    pub class_flags: ClassFlags,
    /// Index of the `Object` class. Not sure if it's ever anything else.
    pub object_class: PackageObjectIndex,
    /// The name of this class's configuration file, as specified using the `config(Name)`
    /// specifier.
    pub config_name: ArchivedName,
    /// The class's subobjects (components,) as created using the `Begin Object .. End Object`
    /// syntax in `defaultproperties` blocks.
    pub subobjects: Vec<Subobject>,
    /// List of implemented interfaces for this class.
    pub implements: Vec<ImplementedInterface>,
    /// Functions that do not have an implementation.
    pub empty_functions: Vec<ArchivedName>,
    /// `dontsortcategories(Name)` specifiers.
    pub non_sorted_categories: Vec<ArchivedName>,
    /// `hidecategories(Name)` specifiers.
    pub hide_categories: Vec<ArchivedName>,
    /// `autoexpandcategories(Name)` specifiers.
    pub auto_expand_categories: Vec<ArchivedName>,
    /// Unknown zero value.
    pub _zero: ConstU32<0>,
    /// `forcescriptorder(true)` specifiers.
    pub force_script_order: Bool32,
    /// `classgroup(Name)` specifiers.
    pub class_groups: Vec<ArchivedName>,

    /// The name of the header the class should be generated into. Specified with `native(Name)`.
    pub native_name: UnrealString,
    /// Always `'None'`.
    // Unfortunately there's no way for us to mark this as always `None` using the type system
    // since archived names' IDs are determined at runtime.
    pub _none: ArchivedName,
    /// The class default object (CDO) of this class. Its name is usually prefixed with `Default__`.
    pub class_default_object: PackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subobject {
    pub name: ArchivedName,
    pub default: PackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImplementedInterface {
    pub interface: PackageObjectIndex,
    pub vftable: OptionalPackageObjectIndex,
}

bitflags! {
    /// Class flags; most are set whenever a class contains some specifier (like `ABSTRACT` comes
    /// from `abstract`.)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClassFlags: u32 {
        // These two seem to be present on all classes. Not sure what they mean.
        const COMMON               = 0x00000012;

        const ABSTRACT             = 0x00000001;
        // 0x00000002 is part of COMMON
        /// This is present when the object has a field marked as `config`.
        const HAS_CONFIG           = 0x00000004;
        const TRANSIENT            = 0x00000008;
        // 0x00000010 is part of COMMON
        /// This is present when a class contains `localized` variables.
        const LOCALIZED            = 0x00000020;
        const NATIVE               = 0x00000080;
        const NO_EXPORT            = 0x00000100;
        const PLACEABLE            = 0x00000200;
        const PER_OBJECT_CONFIG    = 0x00000400;
        const NATIVE_REPLICATION   = 0x00000800;
        const EDIT_INLINE_NEW      = 0x00001000;
        const COLLAPSE_CATEGORIES  = 0x00002000;
        const INTERFACE            = 0x00004000;
        const ALWAYS_LOADED        = 0x00008000;
        /// Present when the object has a field marked with `editinline`.
        const HAS_EDIT_INLINE      = 0x00200000;
        /// This is present when the object has an `array<ActorComponent>` field.
        const HAS_COMPONENTS       = 0x00800000;
        const DEPRECATED           = 0x02000000;
        const HIDE_DROPDOWN        = 0x04000000;
        /// This is present on a vast majority of game classes, but not all of them.
        /// It does not affect the game at all; not sure about it affecting the editor though.
        const GAME_CLASS_UNKNOWN   = 0x08000000;
        const NATIVE_ONLY          = 0x20000000;
        const PER_OBJECT_LOCALIZED = 0x40000000;
        /// This is present when the object has at least one field marked `crosslevelpassive` or
        /// `crosslevelactive`. The meaning of those specifiers is unknown, though.
        const CROSS_LEVEL          = 0x80000000;
    }
}

serializable_bitflags! {
    type ClassFlags;
    validate |flags| {
        if !flags.contains(ClassFlags::COMMON) {
            // In case we find a class without these flags, complain loudly.
            // (I haven't yet.)
            warn!("class flags without COMMON");
        }
    }
}
