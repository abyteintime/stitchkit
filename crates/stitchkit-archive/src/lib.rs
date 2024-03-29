use std::io::{Read, Seek, SeekFrom};

use sections::{DependencyTable, ExportTable, ImportTable, NameTable, Summary};
use stitchkit_core::binary::{self, Deserializer, ResultContextExt};

pub mod compression;
pub mod hat;
pub mod index;
pub mod name;
pub mod sections;
pub mod welder;

#[derive(Debug, Clone)]
pub struct Archive {
    pub summary: Summary,
    pub name_table: NameTable,
    pub export_table: ExportTable,
    pub import_table: ImportTable,
    pub dependency_table: DependencyTable,
    pub decompressed_data: Vec<u8>,
}

impl Archive {
    pub fn deserialize(
        deserializer: &mut Deserializer<impl Read + Seek>,
    ) -> Result<Self, binary::Error> {
        deserializer.seek(SeekFrom::Start(0))?;
        let summary = deserializer
            .deserialize::<Summary>()
            .context("cannot deserialize archive summary")?;
        let decompressed = summary
            .decompress_archive_to_memory(deserializer)
            .context("cannot decompress archive to memory")?;
        let mut deserializer = Deserializer::from_buffer(decompressed.as_slice());
        let name_table = summary
            .deserialize_name_table(&mut deserializer)
            .context("cannot deserialize archive name table")?;
        let export_table = summary
            .deserialize_export_table(&mut deserializer)
            .context("cannot deserialize archive name table")?;
        let import_table = summary
            .deserialize_import_table(&mut deserializer)
            .context("cannot deserialize archive name table")?;
        let dependency_table = summary
            .deserialize_dependency_table(&mut deserializer)
            .context("cannot deserialize archive name table")?;
        Ok(Self {
            summary,
            name_table,
            export_table,
            import_table,
            dependency_table,
            decompressed_data: decompressed,
        })
    }

    pub fn without_decompressed_data(self) -> Self {
        Self {
            decompressed_data: vec![],
            ..self
        }
    }
}
