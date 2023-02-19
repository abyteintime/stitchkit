use std::marker::PhantomData;

use anyhow::{anyhow, bail, Context};
use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    sections::ExportTable,
};
use stitchkit_core::binary::{deserialize, Deserialize};

use crate::Field;

/// Walk a linked list of `Field`s.
pub struct WalkField<'a, X>
where
    X: Deserialize,
{
    export_table: &'a ExportTable,
    archive: &'a [u8],
    current: OptionalPackageObjectIndex,
    _phantom: PhantomData<X>,
}

impl<'a, X> WalkField<'a, X>
where
    X: Deserialize,
{
    pub fn new(
        export_table: &'a ExportTable,
        archive: &'a [u8],
        first: OptionalPackageObjectIndex,
    ) -> Self {
        Self {
            export_table,
            archive,
            current: first,
            _phantom: PhantomData,
        }
    }

    pub fn walk_to_next(&mut self) -> anyhow::Result<OptionalPackageObjectIndex> {
        if let Some(export_index) = self.current.export_index() {
            let object_export = self.export_table.get(export_index).ok_or_else(|| {
                anyhow!(
                    "UField points to an invalid object ({export_index:?} out of bounds of the export table)"
                )
            })?;
            let next_object = object_export.get_serial_data(self.archive);
            let field = deserialize::<Field<X>>(next_object)
                .with_context(|| format!("cannot deserialize linked UField at {export_index:?}"))?;
            let current = self.current;
            self.current = field.next_object;
            Ok(current)
        } else {
            if self.current.is_imported() {
                bail!("UField must not contain references to imported objects");
            }
            Ok(OptionalPackageObjectIndex::none())
        }
    }
}

impl<'a, X> Iterator for WalkField<'a, X>
where
    X: Deserialize,
{
    type Item = anyhow::Result<PackageObjectIndex>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.walk_to_next() {
            Ok(OptionalPackageObjectIndex(option)) => option.map(Ok),
            Err(error) => Some(Err(error)),
        }
    }
}
