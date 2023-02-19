#![allow(clippy::manual_strip)]

use std::io::Read;

use anyhow::Context;
use bitflags::bitflags;

use stitchkit_archive::Archive;
use stitchkit_core::{binary::Deserializer, primitive::ConstU64, serializable_bitflags};

use crate::{
    property::{any::PropertyClasses, defaults::AggregateDefaultProperties},
    Chunk, StructChunkData,
};

#[derive(Debug, Clone)]
pub struct Struct {
    pub chunk: Chunk<(), StructChunkData>,
    pub flags: StructFlags,
    pub _unknown: ConstU64<0>,
    pub default_properties: AggregateDefaultProperties,
}

impl Struct {
    /// Deserialize a struct from an input stream.
    ///
    /// Deserializing a struct needs some contextual information that we don't have in basic
    /// [`Deserialize`], hence it's done using an associated function.
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
    ) -> anyhow::Result<Self> {
        let chunk: Chunk<(), StructChunkData> = deserializer
            .deserialize()
            .context("cannot deserialize field Struct::chunk")?;
        Ok(Self {
            flags: deserializer
                .deserialize()
                .context("cannot deserialize field Struct::flags")?,
            _unknown: deserializer
                .deserialize()
                .context("cannot deserialize field Struct::_unknown")?,
            default_properties: AggregateDefaultProperties::deserialize(
                deserializer,
                archive,
                property_classes,
                chunk.data.first_variable,
            )
            .context("cannot deserialize field Struct::default_properties")?,
            chunk,
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StructFlags: u32 {
    }
}

serializable_bitflags!(StructFlags);
