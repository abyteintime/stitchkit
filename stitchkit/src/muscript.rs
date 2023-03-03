use std::{ffi::OsStr, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use muscript_foundation::{
    errors::Diagnostic,
    source::{SourceFile, SourceFileId, SourceFileSet},
};
use muscript_parsing::{
    lexis::{token::TokenKind, Lexer, TokenStream},
    parsing::{self, ast},
};
use tracing::{debug, error, info};
use walkdir::WalkDir;

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    Lex,
    Parse,
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

    /// Print out statistical information telling you of how many files were processed and how many
    /// of them failed.
    #[clap(short, long)]
    stats: bool,
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
            .with_context(|| format!("cannot read source file at {path:?}"));
        match source {
            Ok(source) => {
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
            Err(error) => error!("{error:?}"),
        }
    }

    debug!("Performing action");
    let stats = perform_action(args.action, &source_file_set)?;
    if !stats.diagnostics.is_empty() {
        eprintln!();
        info!("Finished with the following diagnostics:");
        eprintln!();
        for diagnostic in stats.diagnostics {
            diagnostic.emit_to_stderr(&source_file_set)?;
        }
    }
    if args.stats {
        let num_successful = stats.num_processed - stats.num_failed;
        let success_rate = num_successful as f64 / stats.num_processed as f64;
        eprintln!();
        info!(
            "Stats: {num_successful}/{} ({:.02}%) successful",
            stats.num_processed,
            success_rate * 100.0
        );
    }

    Ok(())
}

struct Stats {
    num_processed: usize,
    num_failed: usize,
    diagnostics: Vec<Diagnostic>,
}

fn perform_action(action: Action, source_file_set: &SourceFileSet) -> anyhow::Result<Stats> {
    let mut num_failed = 0;
    let mut diagnostics = vec![];
    for (id, file) in source_file_set.iter() {
        debug!("Processing: {}", file.filename);
        match perform_action_on_source_file(&action, id, file) {
            Ok(()) => (),
            Err(mut diagnosis) => {
                diagnostics.append(&mut diagnosis);
                num_failed += 1;
            }
        }
    }
    Ok(Stats {
        num_failed,
        num_processed: source_file_set.len(),
        diagnostics,
    })
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
                let token = lexer.next()?;
                println!("{token:?} {:?}", &file.source[token.span.to_range()]);
                if token.kind == TokenKind::EndOfFile {
                    break;
                }
            }
        }
        Action::Parse => {
            // No preprocessor for now unfortunately. Our parser will scream when it sees `.
            let lexer = Lexer::new(id, &file.source);
            let mut parser = parsing::Parser::new(id, &file.source, lexer);
            let file = parser.parse::<ast::File>().map_err(|_| parser.errors)?;
            println!("{file:#?}");
        }
    }
    Ok(())
}
