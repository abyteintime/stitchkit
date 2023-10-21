use std::marker::PhantomData;

use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageObjectIndex},
    sections::ExportTable,
};
use stitchkit_core::binary::{self, deserialize, Deserialize, ErrorKind, ResultContextExt};

/// Walk a linked list of `Field`s.
pub struct WalkList<'a, F, T> {
    export_table: &'a ExportTable,
    archive: &'a [u8],
    current: OptionalPackageObjectIndex,
    get_next: F,
    _phantom: PhantomData<T>,
}

impl<'a, F, T> WalkList<'a, F, T>
where
    F: Fn(&T) -> OptionalPackageObjectIndex,
    T: Deserialize,
{
    pub fn new(
        export_table: &'a ExportTable,
        archive: &'a [u8],
        first: OptionalPackageObjectIndex,
        get_next: F,
    ) -> Self {
        Self {
            export_table,
            archive,
            current: first,
            get_next,
            _phantom: PhantomData,
        }
    }

    pub fn walk_to_next(&mut self) -> Result<Option<(PackageObjectIndex, T)>, binary::Error> {
        if let Some(export_index) = self.current.export_index() {
            let object_export = self.export_table.get(export_index).ok_or_else(|| {
                ErrorKind::Deserialize.make(format!("UField points to an invalid object ({export_index:?} out of bounds of the export table)"))
            })?;
            let next_object = object_export.get_serial_data(self.archive);
            let field = deserialize::<T>(next_object)
                .with_context(|| format!("cannot deserialize linked UField at {export_index:?}"))?;
            let current = self.current;
            self.current = (self.get_next)(&field);
            Ok(current.0.map(|index| (index, field)))
        } else {
            if self.current.is_imported() {
                return Err(ErrorKind::Deserialize
                    .make("UField must not contain references to imported objects"));
            }
            Ok(None)
        }
    }
}

impl<'a, F, T> Iterator for WalkList<'a, F, T>
where
    F: Fn(&T) -> OptionalPackageObjectIndex,
    T: Deserialize,
{
    type Item = Result<(PackageObjectIndex, T), binary::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.walk_to_next().transpose()
    }
}
