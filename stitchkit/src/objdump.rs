use std::{fs::File, io::BufReader, ops::RangeInclusive, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use clap::{Subcommand, ValueEnum};
use stitchkit_archive::{
    index::PackageClassIndex, name::archived_name_table, sections::ObjectExport, Archive,
};
use stitchkit_core::binary::{deserialize, Deserializer};
use stitchkit_reflection_types::{
    property::any::{AnyProperty, PropertyClasses},
    Class, DefaultObject, Enum, Function, State, Struct,
};
use tracing::{debug, error, info, info_span, trace, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ObjectKind {
    /// Deserialize UFunctions.
    Function,
    /// Deserialize UStates.
    State,
    /// Deserialize UClasses.
    Class,
    /// Deserialize all types of UProperties.
    Properties,
    /// Deserialize UEnums.
    Enum,
    /// Deserialize UStructs.
    Struct,
    /// Deserialize default objects (those generated from `defaultproperties`).
    Default,
}

#[derive(Debug, Clone)]
pub enum ObjectIndexRange {
    All,
    Bounded(RangeInclusive<u32>),
}

#[derive(Clone, Subcommand)]
pub enum Objdump {
    /// Deserialize object data exported using `ardump export-all`.
    Read {
        /// The object type to deserialize.
        kind: ObjectKind,

        /// The export indices of the objects to be deserialized.
        objects: Option<Vec<ObjectIndexRange>>,

        /// The archive the objects are contained within.
        #[clap(short, long)]
        archive: PathBuf,

        /// Specify to only deserialize objects whose class is the one specified.
        #[clap(long)]
        filter_by_class: Option<PackageClassIndex>,
    },
}

pub fn objdump(dump: Objdump) -> anyhow::Result<()> {
    match dump {
        Objdump::Read {
            kind,
            objects,
            archive,
            filter_by_class,
        } => {
            info!(filename = ?archive, "Opening archive");
            let file =
                BufReader::new(File::open(&archive).context("cannot open archive for reading")?);
            let mut deserializer =
                Deserializer::new(file).context("cannot open archive for deserialization")?;

            debug!("Reading archive");
            let archive = Archive::deserialize(&mut deserializer).context("cannot read archive")?;

            debug!("Finding external classes in import table");
            let property_classes =
                PropertyClasses::new(&archive.name_table, &archive.import_table)?;
            trace!("Property classes: {property_classes:#?}");

            debug!("Printing objects");
            for range in objects.unwrap_or_else(|| vec![ObjectIndexRange::All]) {
                for index in range.to_range(archive.export_table.exports.len()) {
                    let _span = info_span!("object", index = index + 1).entered();

                    let export = archive
                        .export_table
                        .exports
                        .get(index as usize)
                        .ok_or_else(|| {
                            anyhow!("object index {index} out of bounds (range {range:?})")
                        })?;
                    let &ObjectExport {
                        class_index,
                        object_name,
                        ..
                    } = export;

                    if let Some(filter) = filter_by_class {
                        if class_index != filter {
                            continue;
                        }
                    }

                    let binary = &export.get_serial_data(&archive.decompressed_data);
                    if binary.is_empty() {
                        warn!("Object has no serial data");
                        continue;
                    }

                    archived_name_table::with(&archive.name_table, || {
                        let prefix = format!("{} {object_name:?}", index + 1);
                        match dump_object_of_kind(
                            &prefix,
                            &archive,
                            &property_classes,
                            class_index,
                            binary,
                            kind,
                        ) {
                            Ok(()) => (),
                            Err(err) => {
                                // Remember that println! and error! output to different streams.
                                // We still want something sensible in stdout, rather than piling
                                // a bunch of failures on top of one line.
                                println!("{prefix}: [serialization error]");
                                error!("while dumping object {index} {object_name:?}: {err:?}")
                            }
                        }
                    });
                }
            }
        }
    }

    Ok(())
}

fn dump_object_of_kind(
    prefix: &str,
    archive: &Archive,
    property_classes: &PropertyClasses,
    class_index: PackageClassIndex,
    buffer: &[u8],
    kind: ObjectKind,
) -> anyhow::Result<()> {
    match kind {
        ObjectKind::Function => {
            println!("{prefix}: {:#?}", deserialize::<Function>(buffer)?)
        }
        ObjectKind::State => {
            println!("{prefix}: {:#?}", deserialize::<State>(buffer)?)
        }
        ObjectKind::Class => {
            println!("{prefix}: {:#?}", deserialize::<Class>(buffer)?)
        }
        ObjectKind::Properties => {
            if let Some(property) = AnyProperty::deserialize(
                property_classes,
                class_index,
                &mut Deserializer::from_buffer(buffer),
            )? {
                println!("{prefix}: {property:#?}",);
            }
        }
        ObjectKind::Enum => {
            println!("{prefix}: {:#?}", deserialize::<Enum>(buffer)?)
        }
        ObjectKind::Struct => {
            println!(
                "{prefix}: {:#?}",
                Struct::deserialize(
                    &mut Deserializer::from_buffer(buffer),
                    archive,
                    property_classes
                )?
            )
        }
        ObjectKind::Default => {
            let class_buffer =
                archive
                    .export_table
                    .get(class_index.export_index().ok_or_else(|| {
                        anyhow!("class for default object must be an exported class")
                    })?)
                    .ok_or_else(|| anyhow!("default object has an invalid class index"))?
                    .get_serial_data(&archive.decompressed_data);
            let class = deserialize::<Class>(class_buffer)
                .context("cannot deserialize the default object's class")?;
            println!(
                "{prefix}: {:#?}",
                DefaultObject::deserialize(
                    &mut Deserializer::from_buffer(buffer),
                    archive,
                    property_classes,
                    &class,
                )?
            );
        }
    }
    Ok(())
}

impl ObjectIndexRange {
    fn to_range(&self, object_count: usize) -> RangeInclusive<u32> {
        match self {
            ObjectIndexRange::All => 0..=(object_count as u32).saturating_sub(1),
            ObjectIndexRange::Bounded(range) => range.clone(),
        }
    }
}

impl FromStr for ObjectIndexRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == ".." {
            Ok(Self::All)
        } else if let Some((min, max)) = s.split_once("..") {
            Ok(Self::Bounded(min.parse()?..=max.parse()?))
        } else {
            let single = s.parse()?;
            Ok(Self::Bounded(single..=single))
        }
    }
}
