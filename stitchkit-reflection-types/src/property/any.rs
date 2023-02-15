use std::{io::Read, num::NonZeroU32};

use anyhow::Context;
use stitchkit_archive::{
    index::PackageObjectIndex,
    sections::{NameTableEntry, ObjectImport},
};
use stitchkit_core::binary::ReadExt;

use super::{
    ArrayProperty, ByteProperty, ClassProperty, ComponentProperty, DelegateProperty, IntProperty,
    InterfaceProperty, NameProperty, ObjectProperty, StringProperty, StructProperty,
};

/// Represents any type of property.
#[derive(Debug, Clone)]
pub enum AnyProperty {
    Byte(ByteProperty),
    Int(IntProperty),
    String(StringProperty),
    Name(NameProperty),
    Array(ArrayProperty),
    Object(ObjectProperty),
    Class(ClassProperty),
    Interface(InterfaceProperty),
    Delegate(DelegateProperty),
    Struct(StructProperty),
    Component(ComponentProperty),
}

impl AnyProperty {
    /// Deserializes an `AnyProperty` given [`PropertyClasses`], a class index, and a reader.
    ///
    /// Returns `Ok(Some(any))` when the property is successfully deserialized, `Ok(None)` when
    /// `class_index` is not a known property class, or `Err` when deserialization fails.
    pub fn deserialize(
        property_classes: &PropertyClasses,
        class_index: PackageObjectIndex,
        mut reader: impl Read,
    ) -> anyhow::Result<Option<Self>> {
        let class_index = Some(class_index);
        Ok(match class_index {
            i if i == property_classes.byte_property => Some(Self::Byte(
                reader
                    .deserialize()
                    .context("cannot deserialize byte property")?,
            )),
            i if i == property_classes.int_property => Some(Self::Int(
                reader
                    .deserialize()
                    .context("cannot deserialize int property")?,
            )),
            i if i == property_classes.string_property => Some(Self::String(
                reader
                    .deserialize()
                    .context("cannot deserialize string property")?,
            )),
            i if i == property_classes.name_property => Some(Self::Name(
                reader
                    .deserialize()
                    .context("cannot deserialize name property")?,
            )),
            i if i == property_classes.array_property => Some(Self::Array(
                reader
                    .deserialize()
                    .context("cannot deserialize array property")?,
            )),
            i if i == property_classes.object_property => Some(Self::Object(
                reader
                    .deserialize()
                    .context("cannot deserialize object property")?,
            )),
            i if i == property_classes.class_property => Some(Self::Class(
                reader
                    .deserialize()
                    .context("cannot deserialize class property")?,
            )),
            i if i == property_classes.interface_property => Some(Self::Interface(
                reader
                    .deserialize()
                    .context("cannot deserialize interface property")?,
            )),
            i if i == property_classes.delegate_property => Some(Self::Delegate(
                reader
                    .deserialize()
                    .context("cannot deserialize delegate property")?,
            )),
            i if i == property_classes.struct_property => Some(Self::Struct(
                reader
                    .deserialize()
                    .context("cannot deserialize struct property")?,
            )),
            i if i == property_classes.component_property => Some(Self::Component(
                reader
                    .deserialize()
                    .context("cannot deserialize component property")?,
            )),
            _ => None,
        })
    }
}

/// Contains the object indices of all property classes within the archive.
#[derive(Debug, Clone, Default)]
pub struct PropertyClasses {
    pub byte_property: Option<PackageObjectIndex>,
    pub int_property: Option<PackageObjectIndex>,
    pub string_property: Option<PackageObjectIndex>,
    pub name_property: Option<PackageObjectIndex>,
    pub array_property: Option<PackageObjectIndex>,
    pub object_property: Option<PackageObjectIndex>,
    pub class_property: Option<PackageObjectIndex>,
    pub interface_property: Option<PackageObjectIndex>,
    pub delegate_property: Option<PackageObjectIndex>,
    pub struct_property: Option<PackageObjectIndex>,
    pub component_property: Option<PackageObjectIndex>,
}

impl PropertyClasses {
    /// Extracts property classes from an archive's import and name tables.
    pub fn new(name_table: &[NameTableEntry], import_table: &[ObjectImport]) -> Self {
        let mut result = Self::default();
        for (i, import) in import_table.iter().enumerate() {
            if let (b"Core", b"Class", class_name) = import.resolve_names(name_table) {
                let index = Some(PackageObjectIndex::Imported(
                    NonZeroU32::new(i as u32 + 1).unwrap(),
                ));
                match class_name {
                    b"ByteProperty" => result.byte_property = index,
                    b"IntProperty" => result.int_property = index,
                    b"StrProperty" => result.string_property = index,
                    b"NameProperty" => result.name_property = index,
                    b"ArrayProperty" => result.array_property = index,
                    b"ObjectProperty" => result.object_property = index,
                    b"ClassProperty" => result.class_property = index,
                    b"InterfaceProperty" => result.interface_property = index,
                    b"DelegateProperty" => result.delegate_property = index,
                    b"StructProperty" => result.struct_property = index,
                    b"ComponentProperty" => result.component_property = index,
                    _ => (),
                }
            }
        }
        result
    }
}
