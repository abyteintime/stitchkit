use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
};

use anyhow::Context;
use clap::Subcommand;
use stitchkit_archive::{
    binary::ReadExt,
    sections::{NameTableEntry, ObjectExportDebug, ObjectImportDebug, Summary},
};
use tracing::{debug, info};

#[derive(Clone, Copy, Subcommand)]
pub enum Ardump {
    /// Dump summarising information about the archive (aka the archive header).
    Summary,

    /// Dump the `FName` table.
    Names,
    /// Dump the object export table.
    Exports,
    /// Dump the object import table.
    Imports,

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
                println!("{i}: {:#?}", ObjectExportDebug::new(&name_table, export));
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
                println!("{i}: {:#?}", ObjectImportDebug::new(&name_table, import));
            }
        }
        Ardump::TestDecompression => (),
    }

    Ok(())
}
