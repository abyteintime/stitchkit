use std::io::{Cursor, Read, Write};

use stitchkit_archive::{
    index::OptionalPackageObjectIndex, sections::name_table::common::CommonNames, Archive,
};
use stitchkit_core::binary::{self, Deserializer, ResultContextExt, Serialize, Serializer};
use tracing::warn;

use crate::{
    property::{
        any::PropertyClasses,
        defaults::{DefaultProperties, DefaultPropertiesFormat},
    },
    Object,
};

#[derive(Debug, Clone)]
pub struct DefaultObject {
    pub object: Object<()>,
    pub default_properties: DefaultProperties,
}

impl DefaultObject {
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
        class_index: OptionalPackageObjectIndex,
    ) -> Result<Self, binary::Error> {
        Ok(Self {
            object: deserializer
                .deserialize()
                .context("cannot deserialize field DefaultObject::object")?,
            default_properties: DefaultProperties::deserialize::<()>(
                deserializer,
                archive,
                property_classes,
                class_index,
                DefaultPropertiesFormat::Full,
            )
            .context("cannot deserialize field DefaultObject::default_properties")?,
        })
    }

    pub fn serialize_into(
        &self,
        serializer: &mut Serializer<impl Write>,
        names: &CommonNames,
    ) -> Result<(), binary::Error> {
        self.object.serialize(serializer)?;
        warn!("Trying to serialize a default object but default property serialization is not yet implemented");
        // Just serialize a None to signal there's no default properties.
        names.none.serialize(serializer)?;
        Ok(())
    }

    pub fn serialize(&self, names: &CommonNames) -> Result<Vec<u8>, binary::Error> {
        let mut buffer = vec![];
        self.serialize_into(&mut Serializer::new(Cursor::new(&mut buffer)), names)?;
        Ok(buffer)
    }
}
