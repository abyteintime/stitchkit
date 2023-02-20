#![allow(clippy::manual_strip)]

use std::{io::Read, ops::Deref};

use anyhow::Context;
use bitflags::bitflags;

use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName, Archive};
use stitchkit_core::{binary::Deserializer, serializable_bitflags, Deserialize};

use crate::{
    property::{
        any::PropertyClasses,
        defaults::{DefaultProperties, DefaultPropertiesFormat},
    },
    Chunk,
};

#[derive(Debug, Clone, Deserialize)]
pub struct StructHeader {
    pub chunk: Chunk<ArchivedName>,
    pub flags: StructFlags,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub header: StructHeader,
    pub default_properties: DefaultProperties,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StructFlags: u32 {
        /// `immutable` specifier - this struct uses compact binary serialization.
        const IMMUTABLE = 0x00000030;
    }
}

serializable_bitflags!(StructFlags);

impl Struct {
    /// Deserialize a struct from an input stream.
    ///
    /// Deserializing a struct needs some contextual information that we don't have in basic
    /// [`Deserialize`], hence it's done using an associated function.
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read>,
        archive: &Archive,
        property_classes: &PropertyClasses,
        this_struct: OptionalPackageObjectIndex,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            header: deserializer
                .deserialize()
                .context("cannot deserialize field Struct::header")?,
            default_properties: DefaultProperties::deserialize::<ArchivedName>(
                deserializer,
                archive,
                property_classes,
                this_struct,
                DefaultPropertiesFormat::Full,
            )
            .context("cannot deserialize field Struct::default_properties")?,
        })
    }
}

impl Deref for Struct {
    type Target = StructHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
