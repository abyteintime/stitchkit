use std::{collections::HashMap, ffi::OsStr, path::PathBuf, rc::Rc};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticConfig},
    source::{SourceFile, SourceFileId, SourceFileSet},
};
use muscript_parsing::{
    self, ast,
    lexis::{
        preprocessor::{Definitions, Preprocessor},
        token::TokenKind,
        TokenStream,
    },
    Structured,
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

    /// Print debug notes for diagnostics that have them.
    #[clap(long)]
    diagnostics_debug_info: bool,
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
    let dir_prefix = if args.package.is_file() {
        args.package
            .parent()
            .ok_or_else(|| anyhow!("source file must be located in a directory"))?
            .to_owned()
    } else {
        args.package
    };
    let mut source_file_set = SourceFileSet::new();
    for path in source_file_paths {
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read source file at {path:?}"));
        match source {
            Ok(source) => {
                let pretty_file_name = path
                    .strip_prefix(&dir_prefix)?
                    .to_string_lossy()
                    .into_owned();
                source_file_set.add(SourceFile::new(
                    args.package_name.clone(),
                    pretty_file_name,
                    Rc::from(source),
                ));
            }
            Err(error) => error!("{error:?}"),
        }
    }

    debug!("Performing action");
    let stats = perform_action(
        args.action,
        &source_file_set,
        &Definitions {
            map: HashMap::from_iter([]),
        },
    )?;
    if !stats.diagnostics.is_empty() {
        eprintln!();
        info!("Finished with the following diagnostics:");
        eprintln!();
        for diagnostic in stats.diagnostics {
            diagnostic.emit_to_stderr(
                &source_file_set,
                &DiagnosticConfig {
                    show_debug_info: args.diagnostics_debug_info,
                },
            )?;
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

fn perform_action(
    action: Action,
    source_file_set: &SourceFileSet,
    definitions: &Definitions,
) -> anyhow::Result<Stats> {
    let mut num_failed = 0;
    let mut diagnostics = vec![];
    for (id, file) in source_file_set.iter() {
        debug!("Processing: {}", file.filename);
        match perform_action_on_source_file(&action, id, file, definitions) {
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
    definitions: &Definitions,
) -> Result<(), Vec<Diagnostic>> {
    match action {
        Action::Lex => {
            let mut definitions = definitions.clone();
            let mut lexer = Preprocessor::new(id, Rc::clone(&file.source), &mut definitions);
            loop {
                let token = lexer.next()?;
                println!("{token:?} {:?}", &file.source[token.span.to_range()]);
                if token.kind == TokenKind::EndOfFile {
                    break;
                }
            }
        }
        Action::Parse => {
            let mut definitions = definitions.clone();
            let lexer = Preprocessor::new(id, Rc::clone(&file.source), &mut definitions);
            let mut sink = vec![];
            let mut parser =
                muscript_parsing::Parser::new(id, &file.source, Structured::new(lexer), &mut sink);
            let file = parser.parse::<ast::File>();
            if !sink.is_empty() {
                return Err(sink);
            }
            println!("{file:#?}");
        }
    }
    Ok(())
}
