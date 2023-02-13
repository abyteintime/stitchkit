use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::PathBuf,
};

use anyhow::Context;
use clap::{Subcommand, ValueEnum};
use stitchkit_archive::{name::archived_name_table, sections::Summary};
use stitchkit_core::binary::{deserialize, ReadExt};
use stitchkit_reflection_types::{Class, Function, State};
use tracing::{debug, error, info};

#[derive(Clone, Copy, ValueEnum)]
pub enum ObjectKind {
    /// Deserialize UFunctions.
    Function,
    /// Deserialize UStates.
    State,
    /// Deserialize UClasses.
    Class,
}

#[derive(Clone, Subcommand)]
pub enum Objdump {
    /// Deserialize object data exported using `ardump export-all`.
    Read {
        /// The object type to deserialize.
        kind: ObjectKind,
        /// The files containing objects of this type.
        files: Vec<PathBuf>,
        /// The archive the objects were extracted from.
        ///
        /// This needs to be provided for printing out FNames.
        #[clap(short, long)]
        archive: PathBuf,
    },
}

pub fn objdump(dump: Objdump) -> anyhow::Result<()> {
    match dump {
        Objdump::Read {
            kind,
            files,
            archive,
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

            debug!("Printing objects");
            for file in files {
                let uobject =
                    std::fs::read(&file).with_context(|| format!("cannot read file {file:?}"))?;
                let file_name = file
                    .file_stem()
                    .map(|stem| stem.to_string_lossy())
                    .unwrap_or(std::borrow::Cow::Borrowed("_"));
                print!("{file_name}: ");
                archived_name_table::with(&name_table, || -> anyhow::Result<()> {
                    match dump_object_of_kind(&uobject, kind) {
                        Ok(()) => (),
                        Err(err) => error!("while dumping object {file_name}: {err:?}"),
                    }
                    Ok(())
                })?;
            }
        }
    }

    Ok(())
}

fn dump_object_of_kind(buffer: &[u8], kind: ObjectKind) -> anyhow::Result<()> {
    match kind {
        ObjectKind::Function => {
            println!("{:#?}", deserialize::<Function>(buffer)?)
        }
        ObjectKind::State => {
            println!("{:#?}", deserialize::<State>(buffer)?)
        }
        ObjectKind::Class => {
            println!("{:#?}", deserialize::<Class>(buffer)?)
        }
    }
    Ok(())
}
