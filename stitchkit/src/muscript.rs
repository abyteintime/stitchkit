use std::{ffi::OsStr, path::PathBuf};

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use muscript_foundation::{
    errors::{Diagnostic, Severity},
    source::{SourceFile, SourceFileId, SourceFileSet},
};
use muscript_frontend::lexer::{Lexer, TokenKind};
use tracing::debug;
use walkdir::WalkDir;

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    Lex,
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Directory containing the package sources.
    ///
    /// This directory will be walked to look for any .mu files stored within.
    package: PathBuf,

    /// The name of the package. This is primarily used for error messages.
    #[clap(short, long)]
    package_name: String,

    /// Action to take on the source file set.
    #[clap(subcommand)]
    action: Action,
}

pub fn muscript(args: Args) -> anyhow::Result<()> {
    debug!("Looking for source files");
    let mut source_file_paths = vec![];
    for entry in WalkDir::new(&args.package) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("uc")) {
            source_file_paths.push(path.to_owned());
        }
    }
    debug!(
        ?source_file_paths,
        "{} source files found",
        source_file_paths.len()
    );

    debug!("Building source file set");
    let mut source_file_set = SourceFileSet::new();
    for path in source_file_paths {
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read source file at {path:?}"))?;
        let pretty_file_name = path
            .strip_prefix(&args.package)?
            .to_string_lossy()
            .into_owned();
        source_file_set.add(SourceFile::new(
            args.package_name.clone(),
            pretty_file_name,
            source,
        ));
    }

    debug!("Performing action");
    perform_action(args.action, &source_file_set)?;

    Ok(())
}

fn perform_action(action: Action, source_file_set: &SourceFileSet) -> anyhow::Result<()> {
    let mut failed = false;
    for (id, file) in source_file_set.iter() {
        debug!("Processing: {}", file.filename);
        match perform_action_on_source_file(&action, id, file) {
            Ok(()) => (),
            Err(diagnostics) => {
                for diag in diagnostics {
                    if diag.severity >= Severity::Error {
                        failed = true;
                    }
                    diag.emit_to_stderr(source_file_set)?;
                }
            }
        }
    }
    if failed {
        bail!("the MuScript compiler failed with errors")
    } else {
        Ok(())
    }
}

fn perform_action_on_source_file(
    action: &Action,
    id: SourceFileId,
    file: &SourceFile,
) -> Result<(), Vec<Diagnostic>> {
    match action {
        Action::Lex => {
            let mut lexer = Lexer::new(id, &file.source);
            loop {
                let token = lexer.next_token_include_comments()?;
                println!("{token:?} {:?}", &file.source[token.span.0.clone()]);
                if token.kind == TokenKind::EndOfFile {
                    break;
                }
            }
        }
    }
    Ok(())
}
