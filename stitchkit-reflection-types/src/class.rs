#![allow(clippy::manual_strip)]

use bitflags::bitflags;
use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    name::ArchivedName,
};
use stitchkit_core::{primitive::Bool32, serializable_bitflags, string::UnrealString, Deserialize};
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
    /// Always zero.
    pub unknown_2: u32,
    /// `forcescriptorder(true)` specifiers.
    pub force_script_order: Bool32,
    /// `classgroup(Name)` specifiers.
    pub class_groups: Vec<ArchivedName>,

    /// The name of the header the class should be generated into. Specified with `native(Name)`.
    pub native_name: UnrealString,
    /// Always `'None'`.
    pub unknown_4: ArchivedName,
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClassFlags: u32 {
        const ABSTRACT           = 0x00000001;
        // These two seem to be present on an empty object.
        const COMMON             = 0x00000012;
        // const UNKNOWN_1          = 0x00000002;
        // const UNKNOWN_2          = 0x00000010;
        const NATIVE             = 0x00000080;
        const NATIVE_REPLICATION = 0x00000800;
        const INTERFACE          = 0x00004000;
    }
}

serializable_bitflags! {
    type ClassFlags;
    validate |flags| {
        if !flags.contains(ClassFlags::COMMON) {
            warn!("class flags without COMMON");
        }
    }
}
