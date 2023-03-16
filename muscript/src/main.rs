mod input;

use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{bail, Context};
use clap::Parser;
use muscript_analysis::{Environment, Package};
use muscript_foundation::{
    errors::DiagnosticConfig,
    source::{SourceFile, SourceFileSet},
};
use tracing::{debug, error, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, EnvFilter};
use walkdir::WalkDir;

use crate::input::Input;

#[derive(Debug, Parser)]
pub struct Args {
    /// Directory containing the package sources (one directory above `Classes`).
    ///
    /// The `Classes` directory within will be walked to find source files to compile.
    package: PathBuf,

    /// Game source packages. At least `Core` should be provided here.
    #[clap(short = 'x', long)]
    external: Vec<PathBuf>,

    /// Print debug notes for diagnostics that have them.
    #[clap(long)]
    diagnostics_debug_info: bool,
}

pub fn fallible_main(args: Args) -> anyhow::Result<()> {
    debug!("Looking for main package source files");
    let compiled_sources = list_source_files_in_package(&args.package)?;
    debug!("{} source files found", compiled_sources.len());

    debug!("Looking for external source files");
    let mut external_sources = vec![];
    for (i, external_dir) in args.external.iter().enumerate() {
        external_sources.extend(
            list_source_files_in_package(external_dir)?
                .into_iter()
                .map(|path| (i, path)),
        );
    }
    debug!("{} external source files found", external_sources.len());

    // This is kind of inefficient right now because we also load files that we aren't particularly
    // going to use. Thankfully OS-level caching helps alleviate this a bit, but cold compilations
    // are still quite slow because of this extra step.
    debug!("Building source file set");
    let mut source_file_set = SourceFileSet::new();

    debug!("Loading compiled sources");
    let mut compiled_source_file_ids = HashSet::new();
    for path in compiled_sources {
        let source = read_source_file(&path)?;
        let filename = pretty_file_name(&args.package, &path);
        compiled_source_file_ids.insert(source_file_set.add(SourceFile::new(
            filename,
            path,
            Rc::from(source),
        )));
    }

    debug!("Loading external sources");
    for (i, path) in external_sources {
        let external_source_path = &args.external[i];
        let filename = pretty_file_name(external_source_path, &path);
        let source = read_source_file(&path)?;
        source_file_set.add(SourceFile::new(filename, path, Rc::from(source)));
    }

    debug!("Distilling class names from source file set");
    let mut input = Input::new(&source_file_set);
    let mut env = Environment::new();
    let mut classes_to_compile = vec![];
    for (source_file_id, source_file) in source_file_set.iter() {
        match source_file.class_name() {
            Ok(class_name) => {
                if compiled_source_file_ids.contains(&source_file_id) {
                    let class_id = env.allocate_class_id(class_name);
                    classes_to_compile.push(class_id);
                }
                input.add(class_name, source_file_id)
            }
            Err(error) => error!("Error with file {}: {:?}", source_file.filename, error),
        }
    }

    debug!("Compiling package");
    let compilation_result = Package::compile(&mut env, &input, &classes_to_compile);

    for diagnostic in env.diagnostics() {
        _ = diagnostic.emit_to_stderr(
            &source_file_set,
            &DiagnosticConfig {
                show_debug_info: args.diagnostics_debug_info,
            },
        );
    }

    if let Ok(_package) = compilation_result {
        // TODO: Code generation.
    } else {
        error!("Compilation failed, no packages emitted")
    }

    Ok(())
}

fn list_source_files_in_package(package: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let classes_dir = package.join("Classes");
    if !classes_dir.is_dir() {
        bail!("{classes_dir:?} is not a directory");
    }

    let mut source_file_paths = vec![];
    for entry in WalkDir::new(classes_dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("uc")) {
            source_file_paths.push(path.to_owned());
        }
    }
    Ok(source_file_paths)
}

fn read_source_file(path: &Path) -> anyhow::Result<String> {
    let source_bytes =
        std::fs::read(path).with_context(|| format!("cannot read source file at {path:?}"))?;

    if source_bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 big-endian
        let words: Vec<_> = source_bytes
            .chunks_exact(2)
            .map(|arr| (arr[0] as u16) << 8 | arr[1] as u16)
            .collect();
        String::from_utf16(&words[1..]).context("encoding error in UTF-16 (big-endian) file")
    } else if source_bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 little-endian
        let words: Vec<_> = source_bytes
            .chunks_exact(2)
            .map(|arr| (arr[0]) as u16 | (arr[1] as u16) << 8)
            .collect();
        String::from_utf16(&words[1..]).context("encoding error in UTF-16 (little-endian) file")
    } else {
        // UTF-8
        String::from_utf8(source_bytes).context("encoding error in UTF-8 file")
    }
}

fn pretty_file_name(package_root: &Path, source_file: &Path) -> String {
    let path = source_file
        .strip_prefix(package_root)
        .expect("source_file must start with package_root");
    path.to_string_lossy().into_owned()
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

    let args = Args::parse();

    match fallible_main(args) {
        Ok(_) => (),
        Err(error) => error!("{error:?}"),
    }
}
