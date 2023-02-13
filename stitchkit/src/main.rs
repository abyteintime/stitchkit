mod ardump;
mod objdump;

use std::path::PathBuf;

use ardump::{ardump, Ardump};
use clap::{Parser, Subcommand};
use objdump::{objdump, Objdump};
use tracing::{error, info, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Subcommand)]
enum Command {
    /// Read data from an archive.
    ///
    /// Unreal archive files include .u, .upk, and .umap files. These are all the same format,
    /// just using different extensions for some reason.
    Ardump {
        /// Archive to read from.
        filename: PathBuf,

        /// Which part to dump into stdout.
        #[clap(subcommand)]
        what: Ardump,
    },

    /// Operations on object serial data extracted from archives.
    ///
    /// This data can be obtained using the `ardump export-all` command.
    Objdump {
        /// Operation to perform.
        #[clap(subcommand)]
        what: Objdump,
    },
}

#[derive(Parser)]
struct Args {
    /// Tool to run.
    #[clap(subcommand)]
    command: Command,
}

fn fallible_main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Ardump { filename, what } => ardump(&filename, what)?,
        Command::Objdump { what } => objdump(what)?,
    }

    Ok(())
}

fn main() {
    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_writer(std::io::stderr),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("cannot set default tracing subscriber");

    info!("Stitch toolkit version {}", env!("CARGO_PKG_VERSION"));

    match fallible_main() {
        Ok(_) => (),
        Err(err) => {
            error!("in fallible_main: {err:?}");
        }
    }
}
