use std::{io::Cursor, iter::zip};

use anyhow::{bail, Context};
use stitchkit_core::{
    binary::{Serialize, Serializer},
    string::UnrealString,
    uuid::Uuid,
};
use thiserror::Error;

use crate::{
    hat,
    sections::{
        dependency_table::unlinked::UnlinkedDependencyTable,
        export_table::unlinked::UnlinkedExportTable, GenerationInfo, ImportTable, NameTable,
        ObjectExport, PackageFlags, Summary,
    },
};

/// Archive welder. Assembles an archive from unlinked raw parts.
pub struct Welder<'a> {
    pub name_table: &'a NameTable,
    pub import_table: &'a ImportTable,
    pub export_table: &'a UnlinkedExportTable,
    pub dependency_table: &'a UnlinkedDependencyTable,
}

impl<'a> Welder<'a> {
    pub fn weld(self) -> anyhow::Result<Vec<u8>> {
        let mut summary = Summary {
            file_version: hat::ARCHIVE_FORMAT_VERSION,
            package_group: UnrealString::try_from("None").unwrap(),
            package_flags: PackageFlags::COMMON,
            name_table_len: self.name_table.entries.len() as u32,
            export_table_len: self.export_table.exports.len() as u32,
            import_table_len: self.import_table.imports.len() as u32,
            uuid: Uuid::new_v4(),
            generations: vec![GenerationInfo {
                export_count: self.export_table.exports.len() as u32,
                name_count: self.name_table.entries.len() as u32,
                net_object_count: self.export_table.exports.len() as u32,
            }],
            unknown_4: 0xAAAAAAAA,
            ..Default::default()
        };

        let mut result = Vec::new();
        let mut cursor = Cursor::new(&mut result);

        summary
            .serialize(&mut Serializer::new(&mut cursor))
            .context("cannot serialize initial summary")?;

        summary.name_table_offset = cursor
            .position()
            .try_into()
            .map_err(|_| Error::ArchiveTooBig)?;
        for entry in &self.name_table.entries {
            entry
                .serialize(&mut Serializer::new(&mut cursor))
                .context("cannot serialize name table entry")?;
        }

        summary.import_table_offset = cursor
            .position()
            .try_into()
            .map_err(|_| Error::ArchiveTooBig)?;
        for import in &self.import_table.imports {
            import
                .serialize(&mut Serializer::new(&mut cursor))
                .context("cannot serialize import table entry")?;
        }

        // This is a bit of a kludge to produce a seekfree archive. Maybe in the future this can
        // be optimized to derive the serialized size of the export table and dependency table
        // without actually having to serialize them.

        // Anyways, first we produce temporarily broken exports that we then fix up once we know
        // where objects are placed in the file.
        let mut exports = self
            .export_table
            .exports
            .iter()
            .enumerate()
            .map(|(i, option)| {
                option
                    .as_ref()
                    .map(|unlinked| ObjectExport {
                        class_index: unlinked.class_index,
                        super_index: unlinked.super_index,
                        outer_index: unlinked.outer_index,
                        object_name: unlinked.object_name,
                        archetype: unlinked.archetype,
                        object_flags: unlinked.object_flags,
                        serial_size: unlinked.serial_data.len() as u32,
                        serial_offset: 0,
                        export_flags: unlinked.export_flags,
                        unknown_list: unlinked.unknown_list.clone(),
                        uuid: unlinked.uuid,
                        unknown_flags: unlinked.unknown_flags,
                    })
                    .ok_or(Error::UnsetExport(i + 1))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let export_table_offset = cursor.position();
        summary.export_table_offset = export_table_offset
            .try_into()
            .map_err(|_| Error::ArchiveTooBig)?;
        for export in &exports {
            export
                .serialize(&mut Serializer::new(&mut cursor))
                .context("cannot serialize export table entry")?;
        }

        summary.dependency_table_offset = cursor
            .position()
            .try_into()
            .map_err(|_| Error::ArchiveTooBig)?;
        if self.dependency_table.objects.len() < self.export_table.exports.len() {
            bail!(
                "dependency table does not contain dependencies for objects from {} onward",
                self.dependency_table.objects.len()
            );
        }
        for (i, object) in self.dependency_table.objects.iter().enumerate() {
            if let Some(object) = object {
                object
                    .serialize(&mut Serializer::new(&mut cursor))
                    .context("cannot serialize dependency table entry")?;
            } else {
                Err(Error::UnsetDependency(i + 1))?;
            }
        }

        summary.headers_size = cursor
            .position()
            .try_into()
            .map_err(|_| Error::ArchiveTooBig)?;
        summary.headers_size_mirror = summary.headers_size;

        for (unlinked, export) in zip(&self.export_table.exports, &mut exports) {
            let unlinked = unlinked.as_ref().unwrap();
            export.serial_offset = cursor
                .position()
                .try_into()
                .map_err(|_| Error::ArchiveTooBig)?;
            Serializer::new(&mut cursor)
                .write_bytes(&unlinked.serial_data)
                .context("cannot serialize object data")?;
        }

        // As mentioned before, serialize exports again to update serial offsets.
        cursor.set_position(export_table_offset);
        for export in &exports {
            export
                .serialize(&mut Serializer::new(&mut cursor))
                .context("cannot serialize export table entry")?;
        }

        // Go back to the beginning to serialize the summary again, which now contains
        // up to date offsets.
        cursor.set_position(0);
        summary
            .serialize(&mut Serializer::new(&mut cursor))
            .context("cannot serialize final summary")?;

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("archive is too big (its size exceeds the signed 32-bit integer limit)")]
    ArchiveTooBig,
    #[error("export {0} was reserved but never set")]
    UnsetExport(usize),
    #[error("export {0} was specified but it has no counterpart dependency table entry")]
    UnsetDependency(usize),
}
