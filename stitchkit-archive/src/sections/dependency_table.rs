pub mod unlinked;

use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::Deserializer, Deserialize, Serialize};
use tracing::debug;

use crate::index::OptionalPackageObjectIndex;

use super::Summary;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectDependencies {
    pub dependencies: Vec<OptionalPackageObjectIndex>,
}

#[derive(Debug, Clone)]
pub struct DependencyTable {
    pub objects: Vec<ObjectDependencies>,
}

impl Summary {
    pub fn depends_count(&self) -> u32 {
        self.export_table_len
    }

    pub fn deserialize_dependency_table(
        &self,
        deserializer: &mut Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<DependencyTable> {
        debug!(
            "Deserializing dependency table ({} dependencies at {:08x})",
            self.depends_count(),
            self.dependency_table_offset
        );
        deserializer.seek(SeekFrom::Start(self.dependency_table_offset as u64))?;
        let mut objects = Vec::with_capacity(self.depends_count() as usize);
        for i in 0..self.depends_count() {
            objects.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize dependency {i}"))?,
            );
        }
        Ok(DependencyTable { objects })
    }
}
