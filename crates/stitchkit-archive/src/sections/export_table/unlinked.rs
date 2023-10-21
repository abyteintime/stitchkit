use stitchkit_core::{flags::ObjectFlags, uuid::Uuid};

use crate::{
    index::{ExportIndex, OptionalPackageObjectIndex, PackageClassIndex},
    name::ArchivedName,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkedExport {
    pub class_index: PackageClassIndex,
    pub super_index: OptionalPackageObjectIndex,
    pub outer_index: OptionalPackageObjectIndex,
    pub object_name: ArchivedName,
    pub archetype: OptionalPackageObjectIndex,
    pub object_flags: ObjectFlags,
    pub serial_data: Vec<u8>,
    pub export_flags: u32,
    pub unknown_list: Vec<u32>,
    pub uuid: Uuid,
    pub unknown_flags: u32,
}

#[derive(Debug, Clone, Default)]
pub struct UnlinkedExportTable {
    pub exports: Vec<Option<UnlinkedExport>>,
}

impl UnlinkedExportTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, export: UnlinkedExport) -> ExportIndex {
        // TODO: Error handling here in case there are too many exports.
        let index = ExportIndex(self.exports.len() as u32);
        self.exports.push(Some(export));
        index
    }

    pub fn reserve(&mut self) -> ExportIndex {
        let index = ExportIndex(self.exports.len() as u32);
        self.exports.push(None);
        index
    }

    pub fn set(&mut self, index: ExportIndex, export: UnlinkedExport) {
        self.exports[index.0 as usize] = Some(export);
    }
}
