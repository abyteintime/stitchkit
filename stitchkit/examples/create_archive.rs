// The following example demonstrates how to create a class that extends Actor and is placeable
// in the editor.

use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use stitchkit_archive::{
    index::{OptionalPackageObjectIndex, PackageClassIndex},
    name::archived_name_table,
    sections::{
        dependency_table::unlinked::UnlinkedDependencyTable,
        export_table::unlinked::{UnlinkedExport, UnlinkedExportTable},
        name_table::{builder::NameTableBuilder, common::CommonNames},
        ImportTable, ObjectImport,
    },
    welder::Welder,
    Archive,
};
use stitchkit_core::{
    binary::{serialize, Deserializer},
    flags::ObjectFlags,
    primitive::{Bool32, ConstI16, ConstU16, ConstU32, ConstU64},
};
use stitchkit_reflection_types::{
    property::defaults::DefaultProperties, Chunk, Class, ClassFlags, DefaultObject, Events, Field,
    Object, State, TextBuffer,
};
use tracing::{error, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Parser)]
struct Args {
    output_file: PathBuf,
}

fn fallible_main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut name_table = NameTableBuilder::new();
    let mut import_table = ImportTable::new();
    let mut export_table = UnlinkedExportTable::new();
    let mut dependency_table = UnlinkedDependencyTable::new();

    let names = CommonNames::get_or_insert_into(&mut name_table)?;

    let _ = name_table.get_or_insert("Example")?;

    let name_hat_in_time_game = name_table.get_or_insert("HatinTimeGame")?;
    let name_game_mod = name_table.get_or_insert("GameMod")?;

    let import_core = import_table.push(ObjectImport {
        class_package: names.package.core,
        class_name: names.class.package,
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: names.package.core,
    });
    let import_hat_in_time_game = import_table.push(ObjectImport {
        class_package: names.package.core,
        class_name: names.class.package,
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: name_hat_in_time_game,
    });
    let import_text_buffer = import_table.push(ObjectImport {
        class_package: names.package.core,
        class_name: names.class.class,
        outer_index: import_core.into(),
        object_name: name_table.get_or_insert("TextBuffer")?,
    });
    let import_object = import_table.push(ObjectImport {
        class_package: names.package.core,
        class_name: names.class.class,
        outer_index: import_core.into(),
        object_name: names.class.object,
    });
    let import_game_mod = import_table.push(ObjectImport {
        class_package: names.package.core,
        class_name: names.class.class,
        outer_index: import_hat_in_time_game.into(),
        object_name: name_game_mod,
    });
    let import_game_mod_cdo = import_table.push(ObjectImport {
        class_package: name_hat_in_time_game,
        class_name: name_game_mod,
        outer_index: import_hat_in_time_game.into(),
        object_name: name_table.get_or_insert("Default__GameMod")?,
    });

    let export_mod_class = export_table.reserve();

    let mod_source = TextBuffer {
        object: Object {
            index_in_archive: -1,
            extra: names.none,
        },
        _unknown: ConstU64,
        text: "class aaMod extends GameMod;\r\n".try_into()?,
    };
    let export_mod_source = export_table.push(UnlinkedExport {
        class_index: import_text_buffer.into(),
        super_index: None.into(),
        outer_index: export_mod_class.into(),
        object_name: name_table.get_or_insert("ScriptText")?,
        archetype: None.into(),
        object_flags: ObjectFlags::UNKNOWN_3
            | ObjectFlags::NOT_FOR_CLIENT
            | ObjectFlags::NOT_FOR_SERVER,
        serial_data: serialize(&mod_source)?,
        export_flags: 0,
        unknown_list: vec![],
        uuid: Default::default(),
        unknown_flags: 0,
    });

    let mod_cdo = DefaultObject {
        object: Object {
            index_in_archive: -1,
            extra: (),
        },
        default_properties: DefaultProperties { properties: vec![] },
    };
    let export_mod_cdo = export_table.push(UnlinkedExport {
        class_index: export_mod_class.into(),
        super_index: OptionalPackageObjectIndex::none(),
        outer_index: OptionalPackageObjectIndex::none(),
        object_name: name_table.get_or_insert("Default__aaMod")?,
        archetype: import_game_mod_cdo.into(),
        object_flags: ObjectFlags::DEFAULT
            | ObjectFlags::PUBLIC
            | ObjectFlags::UNKNOWN_1
            | ObjectFlags::UNKNOWN_2
            | ObjectFlags::UNKNOWN_3,
        serial_data: mod_cdo.serialize(&names)?,
        export_flags: 0,
        unknown_list: vec![],
        uuid: Default::default(),
        unknown_flags: 0,
    });

    let mod_class = Class {
        state: State {
            chunk: Chunk {
                field: Field {
                    object: Object {
                        index_in_archive: -1,
                        extra: (),
                    },
                    next_object: None.into(),
                },
                parent_chunk: import_game_mod.into(),
                source_code: export_mod_source.into(),
                first_variable: None.into(),
                _zero: ConstU32,
                line_number: -1,
                file_position: -1,
                file_length: 0,
                bytecode: vec![],
            },
            implements_events: Events::DESTROYED | Events::HIT_WALL | Events::PRE_BEGIN_PLAY,
            _unknown: ConstI16,
            enables_events: Events::GAINED_CHILD,

            function_map: vec![],
        },
        class_flags: ClassFlags::COMMON | ClassFlags::HAS_CONFIG | ClassFlags::HAS_COMPONENTS,
        object_class: import_object.into(),
        config_name: name_table.get_or_insert("Mods")?,
        subobjects: vec![],
        implements: vec![],
        empty_functions: vec![
            name_table.get_or_insert("OnOnlinePartySettingsUpdated")?,
            name_table.get_or_insert("OnMiniMissionTimeLimitSecond")?,
            name_table.get_or_insert("OnMiniMissionGenericEvent")?,
            name_table.get_or_insert("OnMiniMissionCancel")?,
            name_table.get_or_insert("OnMiniMissionFail")?,
            name_table.get_or_insert("OnMiniMissionComplete")?,
            name_table.get_or_insert("OnMiniMissionBegin")?,
            name_table.get_or_insert("OnPlayerEnterCannon")?,
            name_table.get_or_insert("OnGuardCaught")?,
            name_table.get_or_insert("OnGuardAlerted")?,
            name_table.get_or_insert("OnBossPhaseMissed")?,
            name_table.get_or_insert("OnPlayerPressedJumpButton")?,
            name_table.get_or_insert("OnPlayerShoved")?,
            name_table.get_or_insert("OnPreBreakableBreak")?,
            name_table.get_or_insert("OnPawnCombatDeath")?,
            name_table.get_or_insert("OnPostPawnCombatTakeDamage")?,
            name_table.get_or_insert("OnPrePawnCombatTakeDamage")?,
            name_table.get_or_insert("OnAllPlayersDead")?,
            name_table.get_or_insert("OnPlayerDeath")?,
            name_table.get_or_insert("OnPreCheckpointSet")?,
            name_table.get_or_insert("OnCheckpointSet")?,
            name_table.get_or_insert("OnWeaponBadgeUsed")?,
            name_table.get_or_insert("OnAbilityUsed")?,
            name_table.get_or_insert("OnLoadoutChanged")?,
            name_table.get_or_insert("OnStatusEffectRemoved")?,
            name_table.get_or_insert("OnPreStatusEffectAdded")?,
            name_table.get_or_insert("OnPreRestartMap")?,
            name_table.get_or_insert("OnPreActSelectMapChange")?,
            name_table.get_or_insert("OnPreOpenHUD")?,
            name_table.get_or_insert("OnCollectedCollectible")?,
            name_table.get_or_insert("OnCollectibleSpawned")?,
            name_table.get_or_insert("OnTimePieceCollected")?,
            name_table.get_or_insert("OnGetDefaultPlayerClass")?,
            name_table.get_or_insert("OnPostLevelIntro")?,
            name_table.get_or_insert("OnPostInitGame")?,
        ],
        non_sorted_categories: vec![],
        hide_categories: vec![name_table.get_or_insert("Navigation")?],
        auto_expand_categories: vec![],
        _zero: ConstU32,
        force_script_order: false.into(),
        class_groups: vec![],
        native_name: Default::default(),
        _none: names.none,
        class_default_object: export_mod_cdo.into(),
    };
    export_table.set(
        export_mod_class,
        UnlinkedExport {
            class_index: PackageClassIndex::class(),
            super_index: import_game_mod.into(),
            outer_index: OptionalPackageObjectIndex::none(),
            object_name: name_table.get_or_insert("aaMod")?,
            archetype: OptionalPackageObjectIndex::none(),
            object_flags: ObjectFlags::PUBLIC
                | ObjectFlags::UNKNOWN_1
                | ObjectFlags::UNKNOWN_2
                | ObjectFlags::UNKNOWN_3
                | ObjectFlags::STANDALONE,
            serial_data: serialize(&mod_class)?,
            export_flags: 0,
            unknown_list: vec![],
            uuid: Default::default(),
            unknown_flags: 0,
        },
    );

    dependency_table.set(export_mod_source, vec![]);
    dependency_table.set(export_mod_cdo, vec![export_mod_class.into()]);
    dependency_table.set(
        export_mod_class,
        vec![export_mod_cdo.into(), export_mod_source.into()],
    );

    let archive = Welder {
        name_table: &name_table.build().context("cannot build name table")?,
        import_table: &import_table,
        export_table: &export_table,
        dependency_table: &dependency_table,
    }
    .weld()
    .context("cannot weld archive")?;

    std::fs::write(&args.output_file, archive)?;

    // Deserialize the archive again.
    let file = BufReader::new(File::open(&args.output_file)?);
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
