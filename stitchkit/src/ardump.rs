use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
};

use anyhow::Context;
use clap::Subcommand;
use stitchkit_archive::{
    binary::ReadExt,
    sections::{NameTableEntry, Summary},
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
            for (i, NameTableEntry { name, flags }) in name_table.names.iter().enumerate() {
                println!("{i:6} {name:?} (0x{flags:016x})");
            }
        }
        Ardump::Exports => todo!("exports"),
        Ardump::Imports => todo!("imports"),
        Ardump::TestDecompression => (),
    }

    Ok(())
}
