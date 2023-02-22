use crate::index::{ExportIndex, OptionalPackageObjectIndex};

use super::ObjectDependencies;

#[derive(Debug, Clone, Default)]
pub struct UnlinkedDependencyTable {
    pub(crate) objects: Vec<Option<ObjectDependencies>>,
}

impl UnlinkedDependencyTable {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(
        &mut self,
        export: impl Into<ExportIndex>,
        depends_on: Vec<OptionalPackageObjectIndex>,
    ) {
        let index = export.into().0 as usize;
        if index >= self.objects.len() {
            self.objects.resize(index + 1, None);
        }
        self.objects[index] = Some(ObjectDependencies {
            dependencies: depends_on,
        });
    }
}
