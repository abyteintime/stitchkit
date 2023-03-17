use bitflags::bitflags;
use indexmap::IndexMap;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    source::{SourceFileId, SourceFileSet},
};
use muscript_syntax::{cst, lexis::token::Ident};

/// Partitions a class into its individual pieces, but without type information.
///
/// A partition is a data structure that's a level above the CST, but also a level above what a
/// hypothetical AST could look like. It does not represent the syntax of a class, but rather its
/// untyped structure.
///
/// Note that a class may be composed of many partitions, in case the class is declared as
/// `partial`. That's because a single partition corresponds to a single file.
#[derive(Debug, Clone)]
pub struct UntypedClassPartition {
    pub source_file_id: SourceFileId,

    pub kind: cst::ClassKind,
    pub name: Ident,
    pub extends: Option<Ident>,
    pub within: Option<Ident>,

    pub consts: IndexMap<CaseInsensitive<String>, cst::ItemConst>,
    pub vars: IndexMap<CaseInsensitive<String>, cst::ItemVar>,
    pub functions: IndexMap<CaseInsensitive<String>, cst::ItemFunction>,
    pub structs: IndexMap<CaseInsensitive<String>, cst::ItemStruct>,
    pub enums: IndexMap<CaseInsensitive<String>, cst::ItemEnum>,
    pub states: IndexMap<CaseInsensitive<String>, cst::ItemState>,

    pub default_properties: Option<cst::ItemDefaultProperties>,
    pub replication: Option<cst::ItemReplication>,
    // NOTE: cpptext is omitted because MuScript does not support exporting C++ headers.
}

#[derive(Debug, Clone)]
pub struct ClassSpecifiers {
    pub flags: ClassFlags,
    pub auto_expand_categories: Vec<Ident>,
    pub class_group: Vec<Ident>,
    pub config: Option<Ident>,
    pub depends_on: Option<Ident>,
    pub dont_sort_categories: Vec<Ident>,
    pub hide_categories: Vec<Ident>,
    pub implements: Vec<Ident>,
    // inherits and native are omitted because we don't support emitting C++.
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ClassFlags: u16 {
        const ABSTRACT = 0x1;
        const ALWAYS_LOADED = 0x2;
        const COLLAPSE_CATEGORIES = 0x4;
        const DEPRECATED = 0x8;
        const DONT_COLLAPSE_CATEGORIES = 0x10;
        const EDIT_INLINE_NEW = 0x20;
        const FORCE_SCRIPT_ORDER = 0x40;
        const HIDE_DROPDOWN = 0x80;
        const ITERATION_OPTIMIZED = 0x100;
        const NEVER_COOK = 0x200;
        const NO_EXPORT = 0x400;
        const NOT_PLACEABLE = 0x800;
        const PER_OBJECT_CONFIG = 0x1000;
        const PLACEABLE = 0x2000;
        const TRANSIENT = 0x4000;
    }
}

impl UntypedClassPartition {
    pub fn from_cst(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        file: cst::File,
    ) -> Self {
        let source = &sources.get(source_file_id).source;
        let class = file.class;

        macro_rules! name_to_string {
            ($item:expr) => {
                CaseInsensitive::new($item.name.span.get_input(source).to_owned())
            };
        }

        let mut consts = IndexMap::new();
        let mut vars = IndexMap::new();
        let mut functions = IndexMap::new();
        let mut structs = IndexMap::new();
        let mut enums = IndexMap::new();
        let mut states = IndexMap::new();
        let mut default_properties = None;
        let mut replication = None;
        for mut item in file.items {
            item = Self::lower_simulated(item);

            match item {
                cst::Item::Empty(semi) => {
                    diagnostics.emit(
                        Diagnostic::warning(source_file_id, "unnecessary semicolon `;`")
                            .with_label(Label::primary(semi.span, ""))
                            .with_note("note: semicolons are only required after `var` and `const` declarations")
                    );
                }
                cst::Item::Var(item_var) => {
                    for var in Self::lower_var_into_many(item_var) {
                        vars.insert(
                            CaseInsensitive::new(
                                var.variables[0].name.span.get_input(source).to_owned(),
                            ),
                            var,
                        );
                    }
                }
                cst::Item::Const(item_const) => {
                    consts.insert(name_to_string!(item_const), item_const);
                }
                cst::Item::Simulated(_) => unreachable!("handled by lower_simulated earlier"),
                cst::Item::Function(item_function) => {
                    functions.insert(name_to_string!(item_function), item_function);
                }
                cst::Item::Struct(item_struct) => {
                    structs.insert(name_to_string!(item_struct.def), item_struct);
                }
                cst::Item::Enum(item_enum) => {
                    enums.insert(name_to_string!(item_enum.def), item_enum);
                }
                cst::Item::State(item_state) => {
                    states.insert(name_to_string!(item_state), item_state);
                }
                cst::Item::DefaultProperties(item_default_properties) => {
                    default_properties = Some(item_default_properties);
                }
                cst::Item::StructDefaultProperties(item_struct_default_properties) => diagnostics
                    .emit(
                        Diagnostic::error(
                            source_file_id,
                            "`structdefaultproperties` may only appear in structs",
                        )
                        .with_label(Label::primary(
                            item_struct_default_properties.keyword.span,
                            "",
                        )),
                    ),
                cst::Item::Replication(item_replication) => {
                    replication = Some(item_replication);
                }
                cst::Item::CppText(item_cpp_text) => diagnostics.emit(
                    Diagnostic::warning(source_file_id, "`cpptext` item is ignored")
                        .with_label(Label::primary(item_cpp_text.cpptext.span, ""))
                        .with_note("note: MuScript does not support exporting C++ headers"),
                ),
                cst::Item::StructCppText(item_struct_cpp_text) => diagnostics.emit(
                    Diagnostic::error(source_file_id, "`structcpptext` may only appear in structs")
                        .with_label(Label::primary(item_struct_cpp_text.cpptext.span, "")),
                ),
                cst::Item::Stmt(stmt) => {
                    todo!("emit diagnostic when stmt occurs in class");
                }
            }
        }

        Self {
            source_file_id,
            kind: class.class,
            name: class.name,
            extends: class.extends.map(|x| {
                let path = &x.parent_class.components;
                if x.parent_class.components.len() > 1 {
                    diagnostics.emit(
                        Diagnostic::error(source_file_id, "parent class cannot be a path")
                            .with_label(Label::primary(path[1].span, ""))
                            .with_note("help: paths `A.B` are used to refer to items declared within classes, not classes themselves")
                            .with_note(format!("note: from now on assuming you meant to use `{}` as the parent class", path[0].span.get_input(source)))
                    )
                }
                path[0]
            }),
            within: class.within.map(|x| x.outer_class),
            consts,
            vars,
            functions,
            structs,
            enums,
            states,
            default_properties,
            replication,
        }
    }

    fn lower_simulated(item: cst::Item) -> cst::Item {
        if let cst::Item::Simulated(simulated) = item {
            match simulated.item {
                cst::SimulatedItem::Function(mut f) => {
                    f.pre_specifiers
                        .push(cst::FunctionSpecifier::Simulated(Ident {
                            span: simulated.simulated.span,
                        }));
                    cst::Item::Function(f)
                }
                cst::SimulatedItem::State(s) => cst::Item::State(cst::ItemState {
                    simulated: Some(simulated.simulated),
                    ..s
                }),
            }
        } else {
            item
        }
    }

    fn lower_var_into_many(mut var: cst::ItemVar) -> impl Iterator<Item = cst::ItemVar> {
        let variables = std::mem::take(&mut var.variables);
        variables
            .into_iter()
            .map(move |single_variable| cst::ItemVar {
                variables: vec![single_variable],
                ..var.clone()
            })
    }
}
