use std::{
    fs::File,
    io::{BufReader, Cursor},
    ops::RangeInclusive,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Context};
use clap::{Subcommand, ValueEnum};
use stitchkit_archive::{
    index::PackageObjectIndex,
    name::archived_name_table,
    sections::{ObjectExport, Summary},
};
use stitchkit_core::binary::{deserialize, ReadExt};
use stitchkit_reflection_types::{
    property::any::{AnyProperty, PropertyClasses},
    Class, Function, State,
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
        filter_by_class: Option<PackageObjectIndex>,
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
            let mut file =
                BufReader::new(File::open(&archive).context("cannot open archive for reading")?);

            debug!("Reading summary");
            let summary = file
                .deserialize::<Summary>()
                .context("cannot read archive summary")?;
            debug!("Reading entire archive to memory");
            let archive = summary
                .decompress_archive_to_memory(file)
                .context("cannot fully load archive to memory")?;
            let mut reader = Cursor::new(&archive);

            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot read archive name table")?;

            debug!("Reading export table");
            let export_table = summary
                .deserialize_export_table(&mut reader)
                .context("cannot read archive export table")?;

            debug!("Reading import table");
            let import_table = summary
                .deserialize_import_table(&mut reader)
                .context("cannot read archive import table")?;
            debug!("Finding external classes in import table");
            let property_classes = PropertyClasses::new(&name_table, &import_table);
            trace!("Property classes: {property_classes:#?}");

            debug!("Printing objects");
            for range in objects.unwrap_or_else(|| vec![ObjectIndexRange::All]) {
                for index in range.to_range(export_table.len()) {
                    let _span = info_span!("object", index = index + 1).entered();

                    let &ObjectExport {
                        class_index,
                        object_name,
                        serial_offset,
                        serial_size,
                        ..
                    } = export_table.get(index as usize).ok_or_else(|| {
                        anyhow!("object index {index} out of bounds (range {range:?})")
                    })?;

                    if let Some(filter) = filter_by_class {
                        if class_index != filter {
                            continue;
                        }
                    }

                    let (serial_offset, serial_size) =
                        (serial_offset as usize, serial_size as usize);
                    let binary = &archive[serial_offset..serial_offset + serial_size];

                    if binary.is_empty() {
                        warn!("Object has no serial data");
                        continue;
                    }

                    archived_name_table::with(&name_table, || {
                        let prefix = format!("{} {object_name:?}", index + 1);
                        match dump_object_of_kind(
                            &prefix,
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
                                println!("[serialization error]");
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
    property_classes: &PropertyClasses,
    class_index: PackageObjectIndex,
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
            if let Some(property) =
                AnyProperty::deserialize(property_classes, class_index, Cursor::new(buffer))?
            {
                println!("{prefix}: {property:#?}",);
            }
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
