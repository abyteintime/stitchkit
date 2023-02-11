mod ardump;

use std::path::PathBuf;

use ardump::{ardump, Ardump};
use clap::{Parser, Subcommand};
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
        .with(tracing_subscriber::fmt::layer().without_time());
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
