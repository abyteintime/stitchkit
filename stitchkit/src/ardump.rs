use std::{
    fs::File,
    io::{BufReader, Cursor},
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use clap::Subcommand;
use stitchkit_archive::{
    index::PackageObjectIndex,
    name::archived_name_table,
    sections::{NameTableEntry, ObjectExport, Summary},
};
use stitchkit_core::binary::ReadExt;
use tracing::{debug, info};

#[derive(Clone, Subcommand)]
pub enum Ardump {
    /// Dump summarising information about the archive (aka the archive header).
    Summary,

    /// Dump the `FName` table.
    Names,
    /// Dump the object export table.
    Exports,
    /// Dump the object import table.
    Imports,
    /// Dump the object dependency table.
    Depends,

    /// Print information about an export and optionally save the exported object's serial data into
    /// a standalone file.
    Export {
        /// Index of the exported object.
        index: usize,

        /// The file to output to; can be omitted.
        #[clap(short, long)]
        output_file: Option<PathBuf>,
    },
    /// Print information about an import.
    Import {
        /// Index of the imported object.
        index: usize,
    },
    /// Save the serial data of all exported objects into files in a folder.
    ExportAll {
        output_directory: PathBuf,

        /// Specify to only export objects whose class is imported and the imported class is the one
        /// specified by this argument.
        #[clap(long)]
        filter_by_imported_class: Option<NonZeroU32>,
    },

    /// Decompress an archive fully and dump it to disk. NOTE: This does not strip compression
    /// metadata! As such, archives decompressed using this will not be readable. This should only
    /// be used for debugging purposes.
    Decompress { output_file: PathBuf },

    /// Decompress the archive and exit. Used for diagnosing problems with decompression or testing
    /// I/O speed.
    TestDecompression,
}

pub fn ardump(filename: &Path, dump: Ardump) -> anyhow::Result<()> {
    info!(?filename, "Opening archive");
    let mut reader = BufReader::new(File::open(filename)?);

    debug!("Reading summary");
    let summary = reader
        .deserialize::<Summary>()
        .context("cannot deserialize archive summary")?;

    if let Ardump::Summary = dump {
        debug!("Printing summary");
        println!("{:#?}", summary);
        return Ok(());
    }

    debug!("Reading entire archive into memory");
    let archive = summary
        .decompress_archive_to_memory(&mut reader)
        .context("cannot fully load archive to memory")?;
    let mut reader = Cursor::new(&archive);

    match dump {
        Ardump::Summary => unreachable!(),
        Ardump::Names => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;

            debug!("Printing name table");
            for (i, NameTableEntry { name, flags }) in name_table.iter().enumerate() {
                println!("{i:6} {name:?} (0x{flags:016x})");
            }
        }
        Ardump::Exports => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;
            debug!("Reading export table");
            let export_table = summary
                .deserialize_export_table(&mut reader)
                .context("cannot deserialize export table")?;

            debug!("Printing export table");
            for (i, export) in export_table.iter().enumerate() {
                archived_name_table::with(&name_table, || {
                    println!("{}: {:#?}", i + 1, export);
                });
            }
        }
        Ardump::Imports => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;
            debug!("Reading import table");
            let import_table = summary
                .deserialize_import_table(&mut reader)
                .context("cannot deserialize import table")?;

            debug!("Printing import table");
            for (i, import) in import_table.iter().enumerate() {
                archived_name_table::with(&name_table, || {
                    println!("{}: {:#?}", i + 1, import);
                });
            }
        }
        Ardump::Depends => {
            debug!("Reading dependency table");
            let depends_table = summary
                .deserialize_dependency_table(&mut reader)
                .context("cannot deserialize dependency table")?;

            debug!("Printing dependency table");
            for (i, depend) in depends_table.iter().enumerate() {
                println!("{i}: {:?}", depend);
            }
        }
        Ardump::Export { index, output_file } => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;

            debug!("Reading export table");
            let export_table = summary
                .deserialize_export_table(&mut reader)
                .context("cannot deserialize export table")?;

            let export @ &ObjectExport {
                serial_offset,
                serial_size,
                ..
            } = export_table
                .get(index - 1)
                .ok_or_else(|| anyhow!("no object with index {index} found in the export table"))?;
            let (serial_offset, serial_size) = (serial_offset as usize, serial_size as usize);
            let serial_data = &archive[serial_offset..serial_offset + serial_size];

            archived_name_table::with(&name_table, || {
                println!("{:#?}", export);
            });

            if let Some(output_file) = output_file {
                debug!(
                    "Saving {} bytes of object data to {output_file:?}",
                    serial_data.len()
                );
                std::fs::write(output_file, serial_data).context("cannot save object data")?;
            }
        }
        Ardump::Import { index } => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;
            debug!("Reading import table");
            let import_table = summary
                .deserialize_import_table(&mut reader)
                .context("cannot deserialize import table")?;
            let import = import_table
                .get(index - 1)
                .ok_or_else(|| anyhow!("no object with index {index} found in the import table"))?;
            archived_name_table::with(&name_table, || {
                println!("{:#?}", import);
            });
        }
        Ardump::ExportAll {
            output_directory,
            filter_by_imported_class,
        } => {
            debug!("Reading name table");
            let name_table = summary
                .deserialize_name_table(&mut reader)
                .context("cannot deserialize name table")?;

            debug!("Reading export table");
            let export_table = summary
                .deserialize_export_table(&mut reader)
                .context("cannot deserialize export table")?;

            debug!("Saving object files");
            std::fs::create_dir_all(&output_directory).context("cannot create output directory")?;
            for (i, export) in export_table.iter().enumerate() {
                let &ObjectExport {
                    serial_offset,
                    serial_size,
                    class_index,
                    ..
                } = export;
                let (serial_offset, serial_size) = (serial_offset as usize, serial_size as usize);
                let serial_data = &archive[serial_offset..serial_offset + serial_size];

                if let Some(filter) = filter_by_imported_class {
                    if class_index != PackageObjectIndex::Imported(filter) {
                        continue;
                    }
                }

                let filename = format!(
                    "{:04}_{}.uobject",
                    i + 1,
                    name_table[export.object_name.index as usize].name
                );
                debug!("Saving {filename}");
                let path = output_directory.join(&filename);
                std::fs::write(path, serial_data).context("cannot save object data")?;
            }
        }
        Ardump::Decompress { output_file } => {
            debug!("Saving decompressed archive {output_file:?}");
            std::fs::write(output_file, &archive).context("cannot save decompressed archive")?;
        }
        Ardump::TestDecompression => (),
    }

    Ok(())
}
