use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName, Archive};
use stitchkit_core::binary::{
    self, Deserialize, Deserializer, ErrorKind, ResultContextExt, ResultMapToBinaryErrorExt,
    Serialize,
};

use crate::{field::walk::WalkList, Chunk, Field};

use super::any::{AnyProperty, PropertyClasses};

#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: ArchivedName,
    pub property: AnyProperty,
}

pub fn collect_properties<X>(
    archive: &Archive,
    property_classes: &PropertyClasses,
    parent_chunk: OptionalPackageObjectIndex,
) -> Result<Vec<PropertyInfo>, binary::Error>
where
    X: Deserialize + Serialize,
{
    let mut properties = vec![];

    // We need to reverse this order because we want the fields to be collected in order
    // of oldest to youngest ancestor, eg. with (Plane : Vector) we want to visit Vector
    // first and then Plane. This is important for binary-serialized (`immutable`) structs.
    // Unfortunately WalkList is not a DoubleEndedIterator so we can't reverse the iterator
    // itself; we have to collect it to a Vec first.
    let mut classes = WalkList::new(
        &archive.export_table,
        &archive.decompressed_data,
        parent_chunk,
        |chunk: &Chunk<X>| chunk.parent_chunk,
    )
    .collect::<Result<Vec<_>, _>>()
    .context("cannot walk inheritance chain")?;
    classes.reverse();

    for (_, chunk) in classes {
        let class_property_walker = WalkList::new(
            &archive.export_table,
            &archive.decompressed_data,
            chunk.first_variable,
            |field: &Field<ArchivedName>| field.next_object,
        )
        .map(|result| -> Result<_, binary::Error> {
            let (object_index, _) = result?;
            let export = archive
                .export_table
                .try_get(object_index)
                .map_err_to_binary_error(ErrorKind::Deserialize)?;

            AnyProperty::deserialize(
                property_classes,
                export.class_index,
                &mut Deserializer::from_buffer(export.get_serial_data(&archive.decompressed_data)),
            )
            .map(|option| {
                option.map(|property| PropertyInfo {
                    name: export.object_name,
                    property,
                })
            })
        })
        .filter_map(Result::transpose);
        for result in class_property_walker {
            let property_info = result.context("failed to deserialize property from link")?;
            properties.push(property_info);
        }
    }

    Ok(properties)
}
