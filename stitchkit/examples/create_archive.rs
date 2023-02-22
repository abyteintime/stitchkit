// The following example demonstrates how to create a class that extends Actor and is placeable
// in the editor.

use std::{fs::File, io::BufReader};

use anyhow::Context;
use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageClassIndex},
    name::archived_name_table,
    sections::{
        dependency_table::unlinked::UnlinkedDependencyTable,
        export_table::unlinked::{UnlinkedExport, UnlinkedExportTable},
        name_table::builder::NameTableBuilder,
        ImportTable, ObjectImport,
    },
    welder::Welder,
    Archive,
};
use stitchkit_core::{
    binary::{serialize, Deserializer},
    flags::ObjectFlags,
    primitive::{Bool32, ConstI16, ConstU32},
};
use stitchkit_reflection_types::{Chunk, Class, ClassFlags, Events, Field, Object, State};
use tracing::{error, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, EnvFilter};

fn fallible_main() -> anyhow::Result<()> {
    let mut name_table = NameTableBuilder::new();
    let mut import_table = ImportTable::new();
    let mut export_table = UnlinkedExportTable::new();
    let mut dependency_table = UnlinkedDependencyTable::new();

    let name_none = name_table.get_or_insert("None")?;
    let name_core = name_table.get_or_insert("Core")?;
    let name_engine = name_table.get_or_insert("Engine")?;
    let name_package = name_table.get_or_insert("Package")?;
    let name_class = name_table.get_or_insert("Class")?;
    let name_object = name_table.get_or_insert("Object")?;
    let name_actor = name_table.get_or_insert("Actor")?;

    let import_core = import_table.push(ObjectImport {
        class_package: name_core,
        class_name: name_package,
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: name_core,
    });
    let import_engine = import_table.push(ObjectImport {
        class_package: name_core,
        class_name: name_package,
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: name_engine,
    });
    let import_object = import_table.push(ObjectImport {
        class_package: name_core,
        class_name: name_class,
        outer_index: import_core.into(),
        object_name: name_object,
    });
    let import_actor = import_table.push(ObjectImport {
        class_package: name_core,
        class_name: name_class,
        outer_index: import_engine.into(),
        object_name: name_actor,
    });

    let export_test_actor_class = export_table.reserve();

    let test_actor_cdo = vec![];
    let export_test_actor_cdo = export_table.push(UnlinkedExport {
        class_index: PackageClassIndex::class(),
        super_index: OptionalPackageObjectIndex::none(),
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: name_table.get_or_insert("Default__skTestActor")?,
        archetype: OptionalPackageObjectIndex::none(),
        object_flags: ObjectFlags::DEFAULT
            | ObjectFlags::PUBLIC
            | ObjectFlags::UNKNOWN_1
            | ObjectFlags::UNKNOWN_2
            | ObjectFlags::UNKNOWN_3,
        serial_data: test_actor_cdo,
        export_flags: 0,
        unknown_list: vec![],
        uuid: Default::default(),
        unknown_flags: 0,
    });

    let test_actor_class = Class {
        state: State {
            chunk: Chunk {
                field: Field {
                    object: Object {
                        index_in_archive: -1,
                        extra: (),
                    },
                    next_object: OptionalPackageObjectIndex::none(),
                },
                parent_chunk: import_actor.into(),
                source_code: OptionalPackageObjectIndex::none(),
                first_variable: OptionalPackageObjectIndex::none(),
                _zero: ConstU32,
                line_number: -1,
                file_position: -1,
                file_length: 0,
                bytecode: vec![],
            },
            implements_events: Events::default(),
            _unknown: ConstI16,
            enables_events: Events::GAINED_CHILD,
            function_map: vec![],
        },
        class_flags: ClassFlags::PLACEABLE,
        object_class: import_object.into(),
        config_name: name_none,
        subobjects: vec![],
        implements: vec![],
        empty_functions: vec![],
        non_sorted_categories: vec![],
        hide_categories: vec![],
        auto_expand_categories: vec![],
        _zero: ConstU32,
        force_script_order: Bool32::from(false),
        class_groups: vec![],
        native_name: Default::default(),
        _none: name_none,
        class_default_object: export_test_actor_cdo.into(),
    };
    export_table.set(
        export_test_actor_class,
        UnlinkedExport {
            class_index: PackageClassIndex::class(),
            super_index: import_actor.into(),
            outer_index: OptionalPackageObjectIndex::none(),
            object_name: name_table.get_or_insert("skTestActor")?,
            archetype: OptionalPackageObjectIndex::none(),
            object_flags: ObjectFlags::PUBLIC
                | ObjectFlags::UNKNOWN_1
                | ObjectFlags::UNKNOWN_2
                | ObjectFlags::UNKNOWN_3
                | ObjectFlags::STANDALONE,
            serial_data: serialize(&test_actor_class)?,
            export_flags: 0,
            unknown_list: vec![],
            uuid: Default::default(),
            unknown_flags: 0,
        },
    );

    dependency_table.set(export_test_actor_cdo, vec![export_test_actor_class.into()]);
    dependency_table.set(export_test_actor_class, vec![export_test_actor_cdo.into()]);

    let archive = Welder {
        name_table: &name_table.build().context("cannot build name table")?,
        import_table: &import_table,
        export_table: &export_table,
        dependency_table: &dependency_table,
    }
    .weld()
    .context("cannot weld archive")?;

    std::fs::write("Example.u", archive)?;

    // Deserialize the archive again.
    let file = BufReader::new(File::open("Example.u")?);
    let archive = Archive::deserialize(&mut Deserializer::new(file)?)?;
    archived_name_table::with(&archive.name_table, || {
        println!("{:#?}", archive.summary);
        println!("{:#?}", archive.name_table);
        println!("{:#?}", archive.import_table);
        println!("{:#?}", archive.export_table);
        println!("{:#?}", archive.dependency_table);
    });

    Ok(())
}

fn main() {
    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_writer(std::io::stderr),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("cannot set default tracing subscriber");

    match fallible_main() {
        Ok(_) => (),
        Err(err) => {
            error!("in fallible_main: {err:?}");
        }
    }
}
