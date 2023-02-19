//! Serialization of default properties.

use std::io::Read;

use anyhow::{anyhow, bail, Context};
use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName, Archive};
use stitchkit_core::{
    binary::{deserialize, Deserializer},
    primitive::ConstU32,
    string::UnrealString,
    Deserialize,
};
use tracing::{trace, trace_span};

use crate::{field::walk::WalkField, Chunk, StructChunkData};

use super::any::{AnyProperty, PropertyClasses};

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProperty {
    pub name: ArchivedName,
    pub array_index: u32,
    pub value: DefaultPropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AggregateDefaultProperties {
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
    Aggregate(AggregateDefaultProperties),
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

/// Serial property info; this is used by the deserializer to extract data out of nested property
/// types such as `array<int>`.
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    name: ArchivedName,
    property: AnyProperty,
}

pub struct DefaultPropertiesDeserializer<'a, R> {
    archive: &'a Archive,
    property_classes: &'a PropertyClasses,
    property_map: &'a [PropertyInfo],
    input_stream: &'a mut Deserializer<R>,
}

impl<'a, R> DefaultPropertiesDeserializer<'a, R> {
    pub fn new(
        archive: &'a Archive,
        property_classes: &'a PropertyClasses,
        property_list: &'a [PropertyInfo],
        default_property_stream: &'a mut Deserializer<R>,
    ) -> Self {
        Self {
            archive,
            property_classes,
            property_map: property_list,
            input_stream: default_property_stream,
        }
    }

    pub fn next_property(&mut self) -> anyhow::Result<Option<DefaultProperty>>
    where
        R: Read,
    {
        let _span = trace_span!(
            "deserialize_default_property",
            stream_position = self.input_stream.stream_position()
        )
        .entered();
        trace!("Deserializing default property");

        let name = self
            .input_stream
            .deserialize::<ArchivedName>()
            .context("cannot deserialize default property name")?;
        if self.archive.name_table.name_to_str(name) == Some(b"None") {
            return Ok(None);
        }

        let property_info = self
            .property_map
            .iter()
            .find(|info| info.name == name)
            .ok_or_else(|| anyhow!("property {name:?} does not exist"))?;

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

        Ok(Some(DefaultProperty {
            name,
            array_index,
            value: self.deserialize_property_value(&property_info.property, true)?,
        }))
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
                let _struct_type_name: ArchivedName = self
                    .input_stream
                    .deserialize()
                    .context("cannot deserialize a struct type default property's type name")?;
                let struct_type_export = self
                    .archive
                    .export_table
                    .try_get(struct_property.struct_type)
                    .context("struct property contains an invalid struct type")?;
                let chunk = deserialize::<Chunk<(), StructChunkData>>(
                    struct_type_export.get_serial_data(&self.archive.decompressed_data),
                )
                .context("cannot deserialize the struct type's chunk")?;
                let aggregate = AggregateDefaultProperties::deserialize(
                    self.input_stream,
                    self.archive,
                    self.property_classes,
                    chunk.data.first_variable,
                )
                .context("cannot deserialize a struct default property's fields")?;
                DefaultPropertyValue::Aggregate(aggregate)
            }
            other => bail!("unsupported type for default property: {other:#?}",),
        })
    }
}

impl AggregateDefaultProperties {
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
        first_property: OptionalPackageObjectIndex,
    ) -> anyhow::Result<Self> {
        dbg!(first_property);
        let properties = WalkField::<ArchivedName>::new(
            &archive.export_table,
            &archive.decompressed_data,
            first_property,
        )
        .inspect(|x| trace!("{x:?}"))
        .map(|object_index| -> anyhow::Result<_> {
            let object_index = object_index?;
            let export = archive.export_table.try_get(object_index)?;

            AnyProperty::deserialize(
                property_classes,
                export.class_index,
                &mut Deserializer::from_buffer(export.get_serial_data(&archive.decompressed_data)),
            )
            .map(|option| {
                option.map(|property| PropertyInfo {
                    name: export.object_name,
                    property,
                })
            })
        })
        .inspect(|x| trace!("AnyProperty: {x:?}"))
        .filter_map(|result| match result {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
        dbg!(&properties);

        let mut default_properties = vec![];
        let mut deserializer = DefaultPropertiesDeserializer::new(
            archive,
            property_classes,
            &properties,
            deserializer,
        );
        while let Some(property) = deserializer.next_property()? {
            default_properties.push(property);
        }
        Ok(Self {
            properties: default_properties,
        })
    }
}
