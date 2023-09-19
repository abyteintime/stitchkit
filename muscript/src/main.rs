mod input;

use std::{collections::HashSet, path::PathBuf, rc::Rc};

use anyhow::{anyhow, bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use muscript_analysis::{ir::dump::DumpFunction, Compiler, Environment, Package};
use muscript_foundation::{
    errors::{DiagnosticConfig, Severity},
    source::{SourceFile, SourceFileSet},
};
use tracing::{error, info, info_span, metadata::LevelFilter, warn};
use tracing_subscriber::{prelude::*, EnvFilter};
use walkdir::WalkDir;

use crate::input::Input;

#[derive(Debug, Parser)]
pub struct Args {
    /// Directory containing the package sources (one directory above `Classes`).
    ///
    /// The `Classes` directory within will be walked to find source files to compile.
    package: Utf8PathBuf,

    /// External source packages. At least `Core` should be provided here.
    #[clap(short = 's', long)]
    source: Vec<Utf8PathBuf>,

    /// Print debug notes for diagnostics that have them.
    #[clap(long)]
    diagnostics_debug_info: bool,

    /// Do not filter out diagnostics from external packages.
    #[clap(long)]
    diagnostics_external: bool,

    /// Print the analyzed package.
    #[clap(long)]
    dump_analysis_output: bool,

    /// Print function IRs.
    #[clap(long)]
    dump_ir: bool,

    /// Output a performance trace (in Chrome trace event format) to the specified path. https://profiler.firefox.com/
    #[clap(long)]
    trace: Option<PathBuf>,
}

pub fn fallible_main(args: Args) -> anyhow::Result<()> {
    let _span = info_span!("muscript").entered();

    let main_package_name = Rc::from(get_package_name(&args.package)?);
    let compiled_sources = {
        let _span = info_span!("list_main_package_sources", %main_package_name).entered();
        let sources = list_source_files_in_package(&args.package)?;
        info!(source_file_count = sources.len());
        sources
    };

    let (external_sources, package_names) = {
        let _span = info_span!("list_sources_of_external_packages").entered();

        let mut external_sources = vec![];
        let package_names: Vec<Rc<str>> = args
            .source
            .iter()
            .map(|package_path| get_package_name(package_path).map(Rc::from))
            .collect::<Result<Vec<_>, _>>()?;
        for (i, external_dir) in args.source.iter().enumerate() {
            let _span = info_span!(
                "list_external_package_sources",
                package_name = %package_names[i]
            )
            .entered();

            let source_files_in_package = list_source_files_in_package(external_dir)?;
            info!(source_file_count = source_files_in_package.len());
            external_sources.extend(source_files_in_package.into_iter().map(|path| (i, path)));
        }

        info!(source_file_count = external_sources.len());
        (external_sources, package_names)
    };

    // This is kind of inefficient right now because we also load files that we aren't particularly
    // going to use. Thankfully OS-level caching helps alleviate this a bit, but cold compilations
    // are still quite slow because of this extra step.
    let (source_file_set, main_package_source_file_ids) = {
        let _span = info_span!("build_source_file_set").entered();

        let mut source_file_set = SourceFileSet::new();

        let main_package_source_file_ids = {
            let _span = info_span!("load_main_package_sources").entered();

            let mut main_package_source_file_ids = HashSet::new();
            for path in compiled_sources {
                let source = read_source_file(&path)?;
                let filename = pretty_file_name(&args.package, &path);
                main_package_source_file_ids.insert(source_file_set.add(SourceFile::new(
                    Rc::clone(&main_package_name),
                    filename,
                    PathBuf::from(path),
                    Rc::from(source),
                )));
            }
            main_package_source_file_ids
        };

        {
            let _span = info_span!("load_external_sources").entered();

            for (i, path) in external_sources {
                let external_source_path = &args.source[i];
                let package_name = Rc::clone(&package_names[i]);
                let filename = pretty_file_name(external_source_path, &path);
                let source = read_source_file(&path)?;
                source_file_set.add(SourceFile::new(
                    package_name,
                    filename,
                    PathBuf::from(path),
                    Rc::from(source),
                ));
            }
        }

        (source_file_set, main_package_source_file_ids)
    };

    let (input, mut env, classes_to_compile) = {
        let _span = info_span!("compiler_input").entered();

        let mut input = Input::new(&source_file_set);
        let mut env = Environment::new();
        let mut classes_to_compile = vec![];
        for (source_file_id, source_file) in source_file_set.iter() {
            match source_file.class_name() {
                Ok(class_name) => {
                    if main_package_source_file_ids.contains(&source_file_id) {
                        let class_id = env.get_or_create_class(class_name);
                        classes_to_compile.push(class_id);
                    }
                    input.add(class_name, source_file_id)
                }
                Err(error) => error!("Error with file {}: {:?}", source_file.filename, error),
            }
        }
        (input, env, classes_to_compile)
    };

    let compiler = &mut Compiler {
        sources: &source_file_set,
        env: &mut env,
        input: &input,
    };
    let compilation_result = Package::compile(compiler, &classes_to_compile);

    {
        let _span = info_span!("emit_diagnostics").entered();
        for diagnostic in env.diagnostics() {
            let is_from_external_package =
                !main_package_source_file_ids.contains(&diagnostic.source_file);
            if diagnostic.severity >= Severity::Error
                || !is_from_external_package
                || args.diagnostics_external
            {
                _ = diagnostic.emit_to_stderr(
                    &source_file_set,
                    &DiagnosticConfig {
                        show_debug_info: args.diagnostics_debug_info,
                    },
                );
            }
        }
    }

    if let Ok(package) = compilation_result {
        // TODO: Code generation.
        if args.dump_analysis_output {
            let _span = info_span!("dump_analysis_output").entered();
            println!("{env:#?}");
            println!("{package:#?}");
        }
        if args.dump_ir {
            let _span = info_span!("dump_ir").entered();
            for (&class_id, class) in &package.classes {
                println!(
                    "\n{}\n----------------------------------------------------------------",
                    env.class_name(class_id)
                );
                for &function_id in &class.functions {
                    let function = env.get_function(function_id);
                    let ir = env.get_function_ir(function_id);
                    println!(
                        "\n{} {:?}",
                        function.mangled_name,
                        DumpFunction {
                            sources: &source_file_set,
                            env: &env,
                            function,
                            ir,
                        }
                    );
                }
            }
        }
    } else {
        error!("Compilation failed, no packages emitted")
    }

    Ok(())
}

fn get_package_name(package: &Utf8Path) -> anyhow::Result<String> {
    package
        .file_name()
        .ok_or_else(|| anyhow!("path {package:?} has no package name"))
        .map(|package_name| package_name.to_owned())
}

fn list_source_files_in_package(package: &Utf8Path) -> anyhow::Result<Vec<Utf8PathBuf>> {
    let classes_dir = package.join("Classes");
    if !classes_dir.is_dir() {
        bail!("{classes_dir:?} is not a directory");
    }

    let mut source_file_paths = vec![];
    for entry in WalkDir::new(classes_dir) {
        let entry = entry?;
        let path = entry.path();
        if let Some(path) = Utf8Path::from_path(path) {
            if path.is_file() && path.extension() == Some("uc") {
                source_file_paths.push(path.to_owned());
            }
        } else {
            warn!("path contains invalid UTF-8: {path:?}");
        }
    }
    Ok(source_file_paths)
}

fn read_source_file(path: &Utf8Path) -> anyhow::Result<String> {
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

fn pretty_file_name(package_root: &Utf8Path, source_file: &Utf8Path) -> String {
    let package_root = package_root.parent().unwrap_or(package_root);
    source_file
        .strip_prefix(package_root)
        .expect("source_file must start with package_root")
        .to_string()
}

fn main() {
    let args = Args::parse();

    let mut chrome_trace = args.trace.as_ref().map(|trace_path| {
        let (chrome_trace, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(trace_path)
            .include_args(true)
            .build();
        (Some(chrome_trace), guard)
    });

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_writer(std::io::stderr)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::DEBUG.into())
                        .from_env_lossy(),
                ),
        )
        .with(chrome_trace.as_mut().and_then(|(ct, _)| ct.take()));

    tracing::subscriber::set_global_default(subscriber)
        .expect("cannot set default tracing subscriber");

    match fallible_main(args) {
        Ok(_) => (),
        Err(error) => error!("{error:?}"),
    }
}
