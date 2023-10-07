mod coherence;
mod structs;
mod support;

use indexmap::IndexMap;
use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, Note, NoteKind},
    ident::CaseInsensitive,
    source::SourceFileId,
    span::Spanned,
};
use muscript_lexer::{
    sources::LexedSources,
    token::{Token, TokenSpan},
};
use muscript_syntax::{
    cst::{self, NamedItem, TypeOrDef, VarDef},
    token, Spanned,
};
use tracing::info_span;

use crate::{
    diagnostics::{self, notes},
    function::mangling::cst_level::mangled_function_name,
};

pub use structs::*;

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
    pub name: token::Ident,
    pub extends: Option<token::Ident>,
    pub within: Option<token::Ident>,

    // We use IndexMaps so as to preserve the original declaration order, which is important
    // because we don't want our error messages to jump around the file. Instead we want them to go
    // strictly from top to bottom.
    pub vars: IndexMap<CaseInsensitive<String>, VarCst>,
    pub functions: IndexMap<CaseInsensitive<String>, Box<cst::ItemFunction>>,
    pub types: IndexMap<CaseInsensitive<String>, TypeCst>,
    pub states: IndexMap<CaseInsensitive<String>, cst::ItemState>,

    pub default_properties: Option<cst::ItemDefaultProperties>,
    pub replication: Option<cst::ItemReplication>,
    // NOTE: cpptext is omitted because MuScript does not support exporting C++ headers.
}

#[derive(Debug, Clone, Spanned)]
pub struct ItemSingleVar {
    pub var: cst::KVar,
    pub editor: Option<cst::VarEditor>,
    pub specifiers: Vec<cst::VarSpecifier>,
    pub ty: cst::Type,
    pub variable: VarDef,
    pub semi: token::Semi,
}

#[derive(Debug, Clone)]
pub enum VarCst {
    Const(cst::ItemConst),
    Var(ItemSingleVar),
}

#[derive(Debug, Clone)]
pub enum TypeCst {
    Struct(UntypedStruct),
    Enum(cst::EnumDef),
    // Not sure if states belong to the same namespace; AFAIK they're only ever referred to by name
    // (as in name literal) but needs verification.
}

/// # Conversion from CST
impl UntypedClassPartition {
    pub fn from_cst(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &LexedSources<'_>,
        source_file_id: SourceFileId,
        file: cst::File,
    ) -> Self {
        let class = file.class;
        let _span = info_span!(
            "untyped_class_partition_from_cst",
            class_name = sources.source(&class.name),
            class_extends = sources.source(&class.extends)
        )
        .entered();

        let mut vars = IndexMap::new();
        let mut functions = IndexMap::new();
        let mut types = IndexMap::new();
        let mut states = IndexMap::new();

        let mut default_properties = None;
        let mut replication = None;

        for mut item in file.bare.items {
            item = Self::lower_simulated(item);

            match item {
                cst::Item::Empty(semi) => {
                    diagnostics.emit(diagnostics::unnecessary_semicolon(source_file_id, semi).with_note(
                        indoc! {"
                            note: each `var` and `const` declaration needs a single semicolon after it;
                                  having one anywhere else is redundant
                        "},
                    ));
                }
                cst::Item::Var(mut item_var) => {
                    match Self::lower_inline_type_def(&mut item_var.ty) {
                        Some(InlineTypeDef::Enum(enum_def)) => {
                            Self::add_to_scope(
                                diagnostics,
                                sources,
                                source_file_id,
                                &mut types,
                                TypeCst::Enum(enum_def),
                            );
                        }
                        Some(InlineTypeDef::Struct(struct_def)) => {
                            let untyped_struct = UntypedStruct::from_cst(
                                diagnostics,
                                sources,
                                source_file_id,
                                &mut types,
                                struct_def,
                            );
                            Self::add_to_scope(
                                diagnostics,
                                sources,
                                source_file_id,
                                &mut types,
                                TypeCst::Struct(untyped_struct),
                            );
                        }
                        None => (),
                    }
                    for var in ItemSingleVar::lower(item_var) {
                        Self::add_to_scope(
                            diagnostics,
                            sources,
                            source_file_id,
                            &mut vars,
                            VarCst::Var(var),
                        );
                    }
                }
                cst::Item::Const(item_const) => {
                    Self::add_to_scope(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut vars,
                        VarCst::Const(item_const),
                    );
                }
                cst::Item::Simulated(_) => unreachable!("handled by lower_simulated earlier"),
                cst::Item::Function(item_function) => {
                    Self::add_to_scope_with_name(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut functions,
                        Box::new(item_function),
                        |item_function| {
                            mangled_function_name(sources, source_file_id, item_function)
                                .into_owned()
                        },
                    );
                }
                cst::Item::Struct(item_struct) => {
                    let untyped_struct = UntypedStruct::from_cst(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut types,
                        item_struct.def,
                    );
                    Self::add_to_scope(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut types,
                        TypeCst::Struct(untyped_struct),
                    );
                }
                cst::Item::Enum(item_enum) => {
                    Self::add_to_scope(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut types,
                        TypeCst::Enum(item_enum.def),
                    );
                }
                cst::Item::State(item_state) => {
                    Self::add_to_scope(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut states,
                        item_state,
                    );
                }
                cst::Item::DefaultProperties(item_default_properties) => {
                    default_properties = Some(item_default_properties);
                }
                cst::Item::StructDefaultProperties(item_struct_default_properties) => diagnostics
                    .emit(
                        Diagnostic::error("`structdefaultproperties` may only appear in structs")
                            .with_label(Label::primary(
                                &item_struct_default_properties.keyword,
                                "",
                            )),
                    ),
                cst::Item::Replication(item_replication) => {
                    replication = Some(item_replication);
                }
                cst::Item::CppText(item_cpp_text) => diagnostics.emit(
                    Diagnostic::warning("`cpptext` item is ignored")
                        .with_label(Label::primary(&item_cpp_text, ""))
                        .with_note(notes::CPP_UNSUPPORTED),
                ),
                cst::Item::StructCppText(item_struct_cpp_text) => diagnostics.emit(
                    Diagnostic::error("`structcpptext` may only appear in structs")
                        .with_label(Label::primary(&item_struct_cpp_text.cpptext, "")),
                ),
                cst::Item::Stmt(stmt) => {
                    diagnostics.emit(diagnostics::stmt_outside_of_function(
                        source_file_id,
                        stmt.span(),
                    ));
                }
            }
        }

        Self {
            source_file_id,
            kind: class.class,
            name: class.name,
            extends: class.extends.map(|x| {
                let path = &x.parent_class.components;
                if path.len() > 1 {
                    diagnostics.emit(
                        Diagnostic::error("parent class cannot be a path")
                            .with_label(Label::primary(&path[1], ""))
                            .with_note("help: paths `A.B` are used to refer to items declared within classes, not classes themselves")
                            .with_note(format!("note: assuming you meant to use `{}` as the parent class", sources.source(&path[0])))
                    )
                }
                path[0]
            }),
            within: class.within.map(|x| x.outer_class),
            vars,
            functions,
            types,
            states,
            default_properties,
            replication,
        }
    }

    fn add_to_scope<I>(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &LexedSources<'_>,
        source_file_id: SourceFileId,
        scope: &mut IndexMap<CaseInsensitive<String>, I>,
        item: I,
    ) where
        I: NamedItem,
    {
        Self::add_to_scope_with_name(diagnostics, sources, source_file_id, scope, item, |item| {
            sources.source(&item.name()).to_owned()
        })
    }

    fn add_to_scope_with_name<I>(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &LexedSources<'_>,
        source_file_id: SourceFileId,
        scope: &mut IndexMap<CaseInsensitive<String>, I>,
        item: I,
        get_name: impl FnOnce(&I) -> String,
    ) where
        I: NamedItem,
    {
        let name = get_name(&item);
        if let Some(other) = scope.get(CaseInsensitive::new_ref(&name)) {
            diagnostics.emit(
                Self::redeclaration_error(sources, other.name().span, item.name().span).with_note(
                    Note {
                        kind: NoteKind::Debug,
                        text: format!("mangled name of conflicting item: `{name}`"),
                        suggestion: None,
                    },
                ),
            );
        } else {
            scope.insert(CaseInsensitive::new(name.to_owned()), item);
        }
    }

    fn redeclaration_error(
        sources: &LexedSources<'_>,
        span_first: TokenSpan,
        span_re: TokenSpan,
    ) -> Diagnostic<Token> {
        let name_re = sources.source(&span_re);
        let name_first = sources.source(&span_first);

        let mut diagnostic = Diagnostic::error(format!("redefinition of `{name_first}`"))
            .with_label(Label::primary(&span_first, "first defined here"))
            .with_label(Label::primary(&span_re, "redefined here"));

        if name_re != name_first
            && CaseInsensitive::new_ref(name_re) == CaseInsensitive::new_ref(name_first)
        {
            diagnostic = diagnostic.with_note(format!("note: `{name_first}` and `{name_re}` only differ in case, and identifiers in UnrealScript are case-insensitive"));
        }

        diagnostic
            .with_note("help: try renaming one of the items to something different")
            .with_note("note: the first definition is favored over the new one when type checking")
    }

    fn lower_simulated(item: cst::Item) -> cst::Item {
        if let cst::Item::Simulated(simulated) = item {
            match simulated.item {
                cst::SimulatedItem::Function(mut f) => {
                    f.pre_specifiers
                        .push(cst::FunctionSpecifier::Simulated(token::Ident {
                            id: simulated.simulated.id,
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
}

enum InlineTypeDef {
    Struct(cst::StructDef),
    Enum(cst::EnumDef),
}

impl UntypedClassPartition {
    fn lower_inline_type_def(out_ty: &mut TypeOrDef) -> Option<InlineTypeDef> {
        let ty = TypeOrDef::Type(cst::Type {
            specifiers: vec![],
            path: out_ty.path(),
            generic: if let TypeOrDef::Type(ty) = out_ty {
                ty.generic.take()
            } else {
                None
            },
            cpptemplate: None,
        });
        let type_or_def = std::mem::replace(out_ty, ty);
        match type_or_def {
            cst::TypeOrDef::StructDef(struct_def) => Some(InlineTypeDef::Struct(struct_def)),
            cst::TypeOrDef::EnumDef(enum_def) => Some(InlineTypeDef::Enum(enum_def)),
            cst::TypeOrDef::Type(_) => None,
        }
    }
}

impl ItemSingleVar {
    fn lower(mut var: cst::ItemVar) -> impl Iterator<Item = Self> {
        let variables = std::mem::take(&mut var.variables);
        variables.into_iter().map(move |single| Self {
            var: var.var,
            editor: var.editor.clone(),
            specifiers: var.specifiers.clone(),
            ty: match var.ty.clone() {
                TypeOrDef::Type(ty) => ty,
                _ => unreachable!("var type should have been lowered by lower_inline_type_def"),
            },
            variable: single,
            semi: var.semi,
        })
    }
}

impl NamedItem for ItemSingleVar {
    fn name(&self) -> cst::ItemName {
        cst::ItemName {
            span: TokenSpan::single(self.variable.name.id),
        }
    }
}

impl NamedItem for VarCst {
    fn name(&self) -> cst::ItemName {
        match self {
            VarCst::Const(item_const) => item_const.name(),
            VarCst::Var(item_var) => item_var.name(),
        }
    }
}

impl NamedItem for TypeCst {
    fn name(&self) -> cst::ItemName {
        match self {
            TypeCst::Struct(item_struct) => cst::ItemName {
                span: TokenSpan::single(item_struct.name.id),
            },
            TypeCst::Enum(item_enum) => item_enum.name(),
        }
    }
}

pub trait UntypedClassPartitionsExt {
    fn find_var(&self, name: &str) -> Option<(SourceFileId, &VarCst)>;
    fn find_type(&self, name: &str) -> Option<(SourceFileId, &TypeCst)>;

    fn index_of_partition_with_function(&self, name: &str) -> Option<usize>;
}

impl UntypedClassPartitionsExt for &[UntypedClassPartition] {
    fn find_var(&self, name: &str) -> Option<(SourceFileId, &VarCst)> {
        self.iter().find_map(|partition| {
            partition
                .vars
                .get(CaseInsensitive::new_ref(name))
                .map(|cst| (partition.source_file_id, cst))
        })
    }

    fn find_type(&self, name: &str) -> Option<(SourceFileId, &TypeCst)> {
        self.iter().find_map(|partition| {
            partition
                .types
                .get(CaseInsensitive::new_ref(name))
                .map(|cst| (partition.source_file_id, cst))
        })
    }

    fn index_of_partition_with_function(&self, name: &str) -> Option<usize> {
        self.iter().position(|partition| {
            partition
                .functions
                .contains_key(CaseInsensitive::new_ref(name))
        })
    }
}
