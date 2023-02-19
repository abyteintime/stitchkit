use std::io::Read;

use anyhow::Context;
use stitchkit_archive::Archive;
use stitchkit_core::binary::Deserializer;

use crate::{
    property::{any::PropertyClasses, defaults::AggregateDefaultProperties},
    Class, Object,
};

#[derive(Debug, Clone)]
pub struct DefaultObject {
    pub object: Object,
    pub default_properties: AggregateDefaultProperties,
}

impl DefaultObject {
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
        class: &Class,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            object: deserializer
                .deserialize()
                .context("cannot deserialize field DefaultObject::object")?,
            default_properties: AggregateDefaultProperties::deserialize(
                deserializer,
                archive,
                property_classes,
                class.state.chunk.data.first_variable,
            )
            .context("cannot deserialize field DefaultObject::default_properties")?,
        })
    }
}
