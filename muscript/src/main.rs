use std::{collections::HashMap, ffi::OsStr, path::PathBuf, rc::Rc};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticConfig, Severity},
    source::{SourceFile, SourceFileId, SourceFileSet},
};
use muscript_syntax::{
    self, cst,
    lexis::{
        preprocessor::{Definitions, Preprocessor},
        token::TokenKind,
        LexicalContext, PeekCaching, TokenStream,
    },
    Structured,
};
use tracing::{debug, error, info, metadata::LevelFilter, warn};
use tracing_subscriber::{prelude::*, EnvFilter};
use walkdir::WalkDir;

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    Lex,
    Parse {
        /// Parse without printing the AST.
        #[clap(long)]
        no_print: bool,
    },
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

    /// Only print out the first `N` diagnostics.
    #[clap(long, name = "n")]
    diagnostics_limit: Option<usize>,
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
    debug!("{} source files found", source_file_paths.len());

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
        let source_bytes =
            std::fs::read(&path).with_context(|| format!("cannot read source file at {path:?}"));
        let source_bytes = match source_bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{error:?}");
                continue;
            }
        };

        let source = if source_bytes.starts_with(&[0xFE, 0xFF]) {
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
        };

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
    let mut stats = perform_action(
        args.action,
        &source_file_set,
        &Definitions {
            map: HashMap::from_iter([]),
        },
    )?;
    if !stats.diagnostics.is_empty() {
        stats
            .diagnostics
            .sort_by_key(|diagnostic| diagnostic.severity);
        eprintln!();
        info!("Finished with the following diagnostics:");
        eprintln!();
        let limit = args.diagnostics_limit.unwrap_or(10);
        let mut count: usize = 0;
        for diagnostic in stats.diagnostics {
            diagnostic.emit_to_stderr(
                &source_file_set,
                &DiagnosticConfig {
                    show_debug_info: args.diagnostics_debug_info,
                },
            )?;
            count += 1;
            if count >= limit {
                warn!("Only the first {limit} diagnostics are displayed; the limit can be set with `--diagnostics-limit=N`");
                break;
            }
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
                num_failed += diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.severity >= Severity::Error)
                    as usize;
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
            let mut diagnostics = vec![];
            let mut tokens = Preprocessor::new(
                id,
                Rc::clone(&file.source),
                &mut definitions,
                &mut diagnostics,
            );
            loop {
                let token = tokens.next(LexicalContext::Default)?;
                println!("{token:?} {:?}", &file.source[token.span.to_usize_range()]);
                if token.kind == TokenKind::EndOfFile {
                    break;
                }
            }
        }
        Action::Parse { no_print } => {
            let mut definitions = definitions.clone();
            let mut preproc_diagnostics = vec![];
            let tokens = PeekCaching::new(Preprocessor::new(
                id,
                Rc::clone(&file.source),
                &mut definitions,
                &mut preproc_diagnostics,
            ));
            let mut parser_diagnostics = vec![];
            let mut parser = muscript_syntax::Parser::new(
                id,
                &file.source,
                Structured::new(tokens),
                &mut parser_diagnostics,
            );
            let file = parser.parse::<cst::File>();
            preproc_diagnostics.append(&mut parser_diagnostics);
            if !preproc_diagnostics.is_empty() {
                return Err(preproc_diagnostics);
            }
            if !no_print {
                println!("{file:#?}");
            }
        }
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

    let args = Args::parse();

    match muscript(args) {
        Ok(_) => (),
        Err(error) => error!("{error:?}"),
    }
}
