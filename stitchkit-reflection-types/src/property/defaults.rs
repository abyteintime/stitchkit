//! Serialization of default properties.

use std::io::Read;

use anyhow::{anyhow, bail, Context};
use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName, Archive};
use stitchkit_core::{
    binary::{deserialize, Deserialize, Deserializer, Serialize},
    primitive::ConstU32,
    string::UnrealString,
    Deserialize,
};
use tracing::{trace, trace_span};

use crate::{property::collect_properties, StructFlags, StructHeader};

use super::{
    any::{AnyProperty, PropertyClasses},
    PropertyInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProperty {
    pub name: ArchivedName,
    pub array_index: u32,
    pub format: DefaultPropertiesFormat,
    pub value: DefaultPropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProperties {
    pub properties: Vec<DefaultProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultPropertyValue {
    Byte(ByteValue),
    Int(i32),
    Float(f32),
    String(UnrealString),
    Name(ArchivedName),
    Array(Vec<DefaultPropertyValue>),
    Object(OptionalPackageObjectIndex),
    Class(OptionalPackageObjectIndex),
    Delegate(DelegateValue),
    Aggregate(DefaultProperties),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ByteValue {
    Literal(u8),
    Enum(ArchivedName),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DelegateValue {
    pub _unknown: ConstU32<0>,
    pub function_name: ArchivedName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefaultPropertiesFormat {
    /// Full, backwards compatible format which serializes the name, type, and size of each field.
    Full,
    /// Compact format that only serializes values. This is used when a struct has the `immutable`
    /// specifier on it.
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Used in the `Full` format, which doesn't require us to keep any state.
    Full,
    /// Index of current field deserialized with the `Compact` format.
    Compact {
        property_index: usize,
        array_index: u32,
    },
}

pub struct DefaultPropertiesDeserializer<'a, R> {
    archive: &'a Archive,
    property_classes: &'a PropertyClasses,
    property_map: &'a [PropertyInfo],
    input_stream: &'a mut Deserializer<R>,
    state: State,
}

impl<'a, R> DefaultPropertiesDeserializer<'a, R> {
    pub fn new(
        archive: &'a Archive,
        property_classes: &'a PropertyClasses,
        property_list: &'a [PropertyInfo],
        default_property_stream: &'a mut Deserializer<R>,
        format: DefaultPropertiesFormat,
    ) -> Self {
        Self {
            archive,
            property_classes,
            property_map: property_list,
            input_stream: default_property_stream,
            state: match format {
                DefaultPropertiesFormat::Full => State::Full,
                DefaultPropertiesFormat::Compact => State::Compact {
                    property_index: 0,
                    array_index: 0,
                },
            },
        }
    }

    pub fn next_property(&mut self) -> anyhow::Result<Option<DefaultProperty>>
    where
        R: Read,
    {
        let _span =
            trace_span!("default", position = self.input_stream.stream_position()).entered();
        trace!("Deserializing default property");

        match self.state {
            State::Full => {
                let name = self
                    .input_stream
                    .deserialize::<ArchivedName>()
                    .context("cannot deserialize default property name")?;
                if self.archive.name_table.name_to_str(name) == Some(b"None") {
                    return Ok(None);
                }
                let _span = trace_span!("full", ?name).entered();
                let _type_tag = self
                    .input_stream
                    .deserialize::<ArchivedName>()
                    .context("cannot deserialize default property type tag")?;
                let _size = self
                    .input_stream
                    .deserialize::<u32>()
                    .context("cannot deserialize default property size")?;
                let array_index = self
                    .input_stream
                    .deserialize::<u32>()
                    .context("cannot deserialize default property array element")?;

                let property_info = self
                    .property_map
                    .iter()
                    .find(|info| info.name == name)
                    .ok_or_else(|| anyhow!("property {name:?} does not exist"))?;

                Ok(Some(DefaultProperty {
                    name,
                    array_index,
                    format: DefaultPropertiesFormat::Full,
                    value: self
                        .deserialize_property_value(&property_info.property, true)
                        .with_context(|| format!("while deserializing property {name:?}"))?,
                }))
            }
            State::Compact {
                property_index,
                array_index,
            } => {
                if let Some(property_info) = self.property_map.get(property_index) {
                    if array_index >= property_info.property.base().array_length.get() {
                        self.state = State::Compact {
                            property_index: property_index + 1,
                            array_index: 0,
                        };
                    }
                }
                let State::Compact { property_index, array_index } = self.state else { unreachable!() };

                if let Some(property_info) = self.property_map.get(property_index) {
                    let _span = trace_span!("compact", name = ?property_info.name).entered();
                    let value = DefaultProperty {
                        name: property_info.name,
                        array_index,
                        format: DefaultPropertiesFormat::Compact,
                        value: self
                            .deserialize_property_value(&property_info.property, true)
                            .with_context(|| {
                                format!("while deserializing property {:?}", property_info.name)
                            })?,
                    };
                    self.state = State::Compact {
                        property_index,
                        array_index: array_index + 1,
                    };
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn deserialize_property_value(
        &mut self,
        property: &AnyProperty,
        is_top_level: bool,
    ) -> anyhow::Result<DefaultPropertyValue>
    where
        R: Read,
    {
        trace!(
            stream_position = self.input_stream.stream_position(),
            "Deserializing property value {property:#?}",
        );
        Ok(match property {
            AnyProperty::Byte(byte_property) => {
                // Are you fucking kidding me.
                if is_top_level {
                    let _enum_type: ArchivedName = self.input_stream.deserialize().context(
                        "cannot deserialize the name before a byte default property's value",
                    )?;
                }
                DefaultPropertyValue::Byte(if byte_property.enum_object.is_none() {
                    ByteValue::Literal(
                        self
                            .input_stream
                            .deserialize()
                            .context("cannot deserialize a byte default property's value (literal byte expected)")?
                    )
                } else {
                    ByteValue::Enum(
                        self
                            .input_stream
                            .deserialize()
                            .context("cannot deserialize a byte default property's value (enum value name expected)")?
                    )
                })
            }
            AnyProperty::Int(_) => DefaultPropertyValue::Int(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize an int default property's value")?,
            ),
            AnyProperty::Float(_) => DefaultPropertyValue::Float(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize a float default property's value")?,
            ),
            AnyProperty::String(_) => DefaultPropertyValue::String(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize a string default property's value")?,
            ),
            AnyProperty::Name(_) => DefaultPropertyValue::Name(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize a name default property's value")?,
            ),
            AnyProperty::Array(array_property) => {
                let _span = trace_span!("array").entered();
                let len = self
                    .input_stream
                    .deserialize::<u32>()
                    .context("cannot deserialize an array default property's length")?;
                let inner_type_export = self
                    .archive
                    .export_table
                    .try_get(array_property.item_property)
                    .context("array property contains an invalid item type")?;
                let inner_type = AnyProperty::deserialize(
                    self.property_classes,
                    inner_type_export.class_index,
                    &mut Deserializer::from_buffer(
                        inner_type_export.get_serial_data(&self.archive.decompressed_data),
                    ),
                )
                .context("cannot deserialize the inner type of the array")?
                .ok_or_else(|| anyhow!("the inner type of the array is not a property"))?;

                let mut array = Vec::with_capacity(len as usize);
                for i in 0..len {
                    let item = self
                        .deserialize_property_value(&inner_type, false)
                        .with_context(|| format!("cannot deserialize an array default property's element at index {i}"))?;
                    array.push(item);
                }
                DefaultPropertyValue::Array(array)
            }
            AnyProperty::Object(_) | AnyProperty::Component(_) => DefaultPropertyValue::Object(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize an object default property's value")?,
            ),
            AnyProperty::Class(_) => DefaultPropertyValue::Class(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize a class type default property's value")?,
            ),
            AnyProperty::Delegate(_) => DefaultPropertyValue::Delegate(
                self.input_stream
                    .deserialize()
                    .context("cannot deserialize a delegate default property's value")?,
            ),
            AnyProperty::Struct(struct_property) => {
                let _span = trace_span!("struct").entered();
                if self.state == State::Full {
                    let _struct_type_name: ArchivedName = self
                        .input_stream
                        .deserialize()
                        .context("cannot deserialize a struct type default property's type name")?;
                }
                let struct_type_export = self
                    .archive
                    .export_table
                    .try_get(struct_property.struct_type)
                    .context("cannot obtain the type of the struct")?;
                let struct_type = deserialize::<StructHeader>(
                    struct_type_export.get_serial_data(&self.archive.decompressed_data),
                )
                .context("cannot serialize the struct's serial data")?;

                let format = if struct_type.flags.contains(StructFlags::IMMUTABLE) {
                    DefaultPropertiesFormat::Compact
                } else {
                    DefaultPropertiesFormat::Full
                };
                let properties = DefaultProperties::deserialize::<ArchivedName>(
                    self.input_stream,
                    self.archive,
                    self.property_classes,
                    struct_property.struct_type,
                    format,
                )
                .context("cannot deserialize a struct default property's fields")?;

                DefaultPropertyValue::Aggregate(properties)
            }
            other => bail!("unsupported type for default property: {other:#?}",),
        })
    }
}

impl DefaultProperties {
    pub fn deserialize<X>(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
        parent_chunk: OptionalPackageObjectIndex,
        format: DefaultPropertiesFormat,
    ) -> anyhow::Result<Self>
    where
        X: Deserialize + Serialize,
    {
        let properties = collect_properties::<X>(archive, property_classes, parent_chunk)
            .context("cannot collect all properties for type")?;
        trace!("Properties to deserialize: {properties:#?}");

        let mut default_properties = vec![];
        let mut deserializer = DefaultPropertiesDeserializer::new(
            archive,
            property_classes,
            &properties,
            deserializer,
            format,
        );
        while let Some(property) = deserializer.next_property()? {
            default_properties.push(property);
        }
        Ok(Self {
            properties: default_properties,
        })
    }
}
