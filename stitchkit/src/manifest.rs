use std::{
    borrow::Borrow, collections::HashMap, ffi::OsStr, fmt::Write, fs::File, io::BufReader,
    path::PathBuf,
};

use anyhow::{anyhow, ensure, Context};
use clap::Parser;
use stitchkit_archive::{
    index::{ExportIndex, ExportNumber},
    sections::{PackageFlags, Summary},
    Archive,
};
use stitchkit_core::binary::Deserializer;
use stitchkit_manifest::{structure::ManifestFlags, writer::ManifestWriter};
use stitchkit_reflection_types::{Class, ClassFlags};
use tracing::{debug, debug_span, error, trace, warn, Level};
use walkdir::WalkDir;

#[derive(Debug, Parser)]
pub struct Args {
    /// Files or directories containing packages from which the manifest should be generated.
    ///
    /// Directories will be searched recursively for .u files.
    search_packages: Vec<PathBuf>,

    /// Use this flag before a directory to force a non-recursive search.
    ///
    /// This is useful for speeding up searches if you precisely know where you want to look for
    /// package files.
    #[clap(short = 'F', long)]
    flat: Vec<PathBuf>,

    /// Path where the manifest should be written.
    ///
    /// This is usually `$GAME_DIR/HatinTimeGame/Script/Manifest.txt`.
    #[clap(short, long)]
    output: PathBuf,
}

pub fn manifest(args: Args) -> anyhow::Result<()> {
    let mut package_paths = vec![];

    debug!(
        "Searching for packages in {} provided paths (+ {} flat)",
        args.search_packages.len(),
        args.flat.len()
    );

    for path in args.search_packages {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("u")) {
                package_paths.push(path.to_owned());
            }
        }
    }
    for path in args.flat {
        for entry in std::fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() && path.extension() == Some(OsStr::new("u")) {
                package_paths.push(path);
            }
        }
    }
    debug!("{} .u files before filtering", package_paths.len());

    let mut summaries = {
        let _span = debug_span!("load_package_summaries").entered();
        package_paths
            .into_iter()
            .map(|path| -> anyhow::Result<_> {
                let package_name = path
                    .file_stem()
                    .ok_or_else(|| anyhow!("{path:?} does not contain a file stem"))?
                    .to_string_lossy()
                    .into_owned();
                debug!("Loading summary for {package_name}");
                let file = BufReader::new(
                    File::open(&path)
                        .with_context(|| format!("cannot open package at {path:?}"))?,
                );
                let mut deserializer = Deserializer::new(file)?;
                let summary = deserializer.deserialize::<Summary>()?;
                Ok((path, package_name, summary, deserializer))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    // We want to perform this step because compiled scripts are 99.999% more likely to be up
    // to date with the source code, unless someone did a certain amount of tomfoolery to trick us.
    debug!("Rejecting cooked packages over uncooked script packages");
    {
        let mut has_ambiguous_duplicates = false;
        summaries.sort_unstable_by(|(_, a, _, _), (_, b, _, _)| a.cmp(b));
        summaries.dedup_by(|(path_a, name_a, summary_a, _), (path_b, name_b, summary_b, _)| {
            if name_a == name_b && summary_a.package_flags == summary_b.package_flags {
                error!("Packages at {path_a:?} and {path_b:?} have the same set of flags (both are either uncooked or cooked) which means they are ambiguous");
                has_ambiguous_duplicates = true;
                false
            } else {
                name_a == name_b
                    && summary_a.package_flags.contains(PackageFlags::COOKED)
                    && !summary_b.package_flags.contains(PackageFlags::COOKED)
            }
        });
        ensure!(
            !has_ambiguous_duplicates,
            "loaded two packages with the same name and both were cooked or uncooked"
        );
    }
    debug!("{} packages remain", summaries.len());
    if tracing::enabled!(Level::TRACE) {
        summaries
            .iter()
            .for_each(|(_, name, summary, _)| trace!("{name}: {summary:#?}"))
    }

    debug!("Assembling inheritance tree");

    #[derive(Debug, Default)]
    struct ClassInfo {
        package: String,
        flags: ClassFlags,
        groups: Vec<Vec<u8>>,
    }

    let mut inheritance_tree = HashMap::<Vec<u8>, Vec<Vec<u8>>>::new();
    let mut class_info = HashMap::<Vec<u8>, ClassInfo>::new();
    let mut root_class = None;
    for (_, package_name, _, mut deserializer) in summaries {
        let _span = debug_span!("collect_objects", package = package_name).entered();
        let archive = Archive::deserialize(&mut deserializer)
            .with_context(|| format!("cannot deserialize package {package_name}"))?;
        for (i, export) in archive
            .export_table
            .exports
            .iter()
            .enumerate()
            .filter(|(_, export)| export.class_index.is_class())
        {
            let export_number = ExportNumber::from(ExportIndex(i as u32));
            let class_name = archive
                .name_table
                .name_to_str(export.object_name)
                .ok_or_else(|| anyhow!("{export_number:?} contains an invalid class name"))?;
            let super_class_name = if let Some(import_index) = export.super_index.import_index() {
                archive
                    .import_table
                    .get(import_index)
                    .map(|import| import.object_name)
            } else if let Some(export_index) = export.super_index.export_index() {
                archive
                    .export_table
                    .get(export_index)
                    .map(|export| export.object_name)
            } else {
                None
            };
            let super_class_name = super_class_name
                .and_then(|archived_name| archive.name_table.name_to_str(archived_name));
            // The super class name is None in case of Object, which is the root of the inheritance
            // hierarchy and therefore does not have a super class.
            if let Some(super_class_name) = super_class_name {
                trace!(
                    "Read class {} : {}",
                    String::from_utf8_lossy(class_name),
                    String::from_utf8_lossy(super_class_name)
                );
                if super_class_name != class_name {
                    let descendants = inheritance_tree
                        .entry(super_class_name.to_owned())
                        .or_default();
                    descendants.push(class_name.to_owned());
                } else {
                    warn!(
                        "Class {} has itself as its own super class",
                        String::from_utf8_lossy(class_name)
                    )
                }
            } else {
                ensure!(
                    root_class.is_none(),
                    "there must only be a single root class"
                );
                root_class = Some(class_name.to_owned());
            }
            let class = export
                .deserialize_serial_data::<Class>(&archive.decompressed_data)
                .with_context(|| {
                    format!("cannot deserialize serial data for class {export_number:?}")
                })?;
            class_info.insert(
                class_name.to_owned(),
                ClassInfo {
                    package: package_name.clone(),
                    flags: class.class_flags,
                    groups: class
                        .class_groups
                        .into_iter()
                        .flat_map(|name| {
                            archive
                                .name_table
                                .name_to_str(name)
                                .map(|name| name.to_owned())
                        })
                        .collect(),
                },
            );
        }
    }
    debug!("{} classes total", class_info.len());

    debug!("Sorting classes so that they look pretty in the editor");
    for children in inheritance_tree.values_mut() {
        children.sort();
    }

    debug!("Writing manifest");

    fn write_entry_rec(
        writer: &mut ManifestWriter<impl Write>,
        inheritance_tree: &HashMap<Vec<u8>, Vec<Vec<u8>>>,
        class_info: &HashMap<Vec<u8>, ClassInfo>,
        class_name: &[u8],
    ) -> anyhow::Result<()> {
        trace!("Writing class {}", String::from_utf8_lossy(class_name));
        let info = class_info.get(class_name).unwrap();
        let mut flags = ManifestFlags::default();
        if info.flags.contains(ClassFlags::PLACEABLE) {
            flags |= ManifestFlags::PLACEABLE;
        }
        if info.flags.contains(ClassFlags::DEPRECATED) {
            flags |= ManifestFlags::DEPRECATED;
        }
        if info.flags.contains(ClassFlags::ABSTRACT) {
            flags |= ManifestFlags::ABSTRACT;
        }
        let groups = info
            .groups
            .iter()
            .map(|group| String::from_utf8_lossy(group))
            .collect::<Vec<_>>();
        writer.write_entry(stitchkit_manifest::writer::Entry {
            class: &String::from_utf8_lossy(class_name),
            package: &info.package,
            flags,
            groups: groups.iter().map(|x| x.borrow()),
        })?;
        writer.descend();
        for child in inheritance_tree.get(class_name).unwrap_or(&vec![]) {
            write_entry_rec(writer, inheritance_tree, class_info, child)?;
        }
        writer.ascend();
        Ok(())
    }

    let root_class = root_class.ok_or_else(|| {
        anyhow!("no root class found in archives. perhaps you're missing Core.u?")
    })?;
    let mut manifest = String::new();
    let mut writer = ManifestWriter::new(&mut manifest)?;

    write_entry_rec(&mut writer, &inheritance_tree, &class_info, &root_class)?;

    debug!("Saving manifest");
    std::fs::write(&args.output, manifest).context("cannot write output file")?;

    Ok(())
}
