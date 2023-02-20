use std::io::Read;

use anyhow::{anyhow, Context};
use stitchkit_archive::{
    index::PackageClassIndex,
    sections::{ImportTable, NameTable},
};
use stitchkit_core::binary::Deserializer;

use crate::Property;

use super::{
    ArrayProperty, ByteProperty, ClassProperty, ComponentProperty, DelegateProperty, FloatProperty,
    IntProperty, InterfaceProperty, NameProperty, ObjectProperty, StringProperty, StructProperty,
};

/// Represents any type of property.
#[derive(Debug, Clone)]
pub enum AnyProperty {
    Byte(ByteProperty),
    Int(IntProperty),
    Float(FloatProperty),
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
    pub fn base(&self) -> &Property {
        match self {
            AnyProperty::Byte(p) => &p.base,
            AnyProperty::Int(p) => &p.base,
            AnyProperty::Float(p) => &p.base,
            AnyProperty::String(p) => &p.base,
            AnyProperty::Name(p) => &p.base,
            AnyProperty::Array(p) => &p.base,
            AnyProperty::Object(p) => &p.base,
            AnyProperty::Class(p) => &p.base,
            AnyProperty::Interface(p) => &p.base,
            AnyProperty::Delegate(p) => &p.base,
            AnyProperty::Struct(p) => &p.base,
            AnyProperty::Component(p) => &p.base,
        }
    }

    /// Deserializes an `AnyProperty` given [`PropertyClasses`], a class index, and a deserializer.
    ///
    /// Returns `Ok(Some(any))` when the property is successfully deserialized, `Ok(None)` when
    /// `class_index` is not a known property class, or `Err` when deserialization fails.
    pub fn deserialize(
        property_classes: &PropertyClasses,
        class_index: PackageClassIndex,
        deserializer: &mut Deserializer<impl Read>,
    ) -> anyhow::Result<Option<Self>> {
        let class_index = Some(class_index);
        Ok(match class_index {
            i if i == property_classes.byte_property => Some(Self::Byte(
                deserializer
                    .deserialize()
                    .context("cannot deserialize byte property")?,
            )),
            i if i == property_classes.int_property => Some(Self::Int(
                deserializer
                    .deserialize()
                    .context("cannot deserialize int property")?,
            )),
            i if i == property_classes.float_property => Some(Self::Float(
                deserializer
                    .deserialize()
                    .context("cannot deserialize float property")?,
            )),
            i if i == property_classes.string_property => Some(Self::String(
                deserializer
                    .deserialize()
                    .context("cannot deserialize string property")?,
            )),
            i if i == property_classes.name_property => Some(Self::Name(
                deserializer
                    .deserialize()
                    .context("cannot deserialize name property")?,
            )),
            i if i == property_classes.array_property => Some(Self::Array(
                deserializer
                    .deserialize()
                    .context("cannot deserialize array property")?,
            )),
            i if i == property_classes.object_property => Some(Self::Object(
                deserializer
                    .deserialize()
                    .context("cannot deserialize object property")?,
            )),
            i if i == property_classes.class_property => Some(Self::Class(
                deserializer
                    .deserialize()
                    .context("cannot deserialize class property")?,
            )),
            i if i == property_classes.interface_property => Some(Self::Interface(
                deserializer
                    .deserialize()
                    .context("cannot deserialize interface property")?,
            )),
            i if i == property_classes.delegate_property => Some(Self::Delegate(
                deserializer
                    .deserialize()
                    .context("cannot deserialize delegate property")?,
            )),
            i if i == property_classes.struct_property => Some(Self::Struct(
                deserializer
                    .deserialize()
                    .context("cannot deserialize struct property")?,
            )),
            i if i == property_classes.component_property => Some(Self::Component(
                deserializer
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
    pub byte_property: Option<PackageClassIndex>,
    pub int_property: Option<PackageClassIndex>,
    pub float_property: Option<PackageClassIndex>,
    pub string_property: Option<PackageClassIndex>,
    pub name_property: Option<PackageClassIndex>,
    pub array_property: Option<PackageClassIndex>,
    pub object_property: Option<PackageClassIndex>,
    pub class_property: Option<PackageClassIndex>,
    pub interface_property: Option<PackageClassIndex>,
    pub delegate_property: Option<PackageClassIndex>,
    pub struct_property: Option<PackageClassIndex>,
    pub component_property: Option<PackageClassIndex>,
}

impl PropertyClasses {
    /// Extracts property classes from an archive's import and name tables.
    pub fn new(name_table: &NameTable, import_table: &ImportTable) -> anyhow::Result<Self> {
        let mut result = Self::default();
        for (i, import) in import_table.imports.iter().enumerate() {
            if let (b"Core", b"Class", class_name) = import.resolve_names(name_table) {
                let index = Some(PackageClassIndex::new(-i32::try_from(i + 1).map_err(
                    |_| {
                        anyhow!(
                            "import table has too many imports (the count must not exceed i32::MAX)"
                        )
                    },
                )?));
                match class_name {
                    b"ByteProperty" => result.byte_property = index,
                    b"IntProperty" => result.int_property = index,
                    b"FloatProperty" => result.float_property = index,
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
        Ok(result)
    }
}
