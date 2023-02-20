use std::io::Read;

use anyhow::Context;
use stitchkit_archive::{index::OptionalPackageObjectIndex, Archive};
use stitchkit_core::binary::Deserializer;

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
    ) -> anyhow::Result<Self> {
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
}
