use std::io::{Read, Seek, SeekFrom};

use anyhow::Context;
use stitchkit_core::{binary::Deserializer, Deserialize};
use tracing::debug;

use crate::index::PackageObjectIndex;

use super::Summary;

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectDependencies {
    pub dependencies: Vec<PackageObjectIndex>,
}

impl Summary {
    pub fn depends_count(&self) -> u32 {
        self.export_count
    }

    pub fn deserialize_dependency_table(
        &self,
        mut deserializer: Deserializer<impl Read + Seek>,
    ) -> anyhow::Result<Vec<ObjectDependencies>> {
        debug!(
            "Deserializing dependency table ({} dependencies at {:08x})",
            self.depends_count(),
            self.depends_offset
        );
        deserializer.seek(SeekFrom::Start(self.depends_offset as u64))?;
        let mut depends = Vec::with_capacity(self.depends_count() as usize);
        for i in 0..self.depends_count() {
            depends.push(
                deserializer
                    .deserialize()
                    .with_context(|| format!("cannot deserialize dependency {i}"))?,
            );
        }
        Ok(depends)
    }
}
