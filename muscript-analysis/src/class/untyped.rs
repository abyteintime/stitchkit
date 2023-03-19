mod coherence;

use indexmap::IndexMap;
use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, Note, NoteKind},
    ident::CaseInsensitive,
    source::{SourceFileId, SourceFileSet, Span, Spanned},
};
use muscript_syntax::{
    cst::{self, NamedItem, TypeOrDef, VarDef},
    lexis::token,
    Spanned,
};

use crate::{diagnostics::notes, function::mangling::mangled_function_name};

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
    pub functions: IndexMap<CaseInsensitive<String>, cst::ItemFunction>,
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
    Struct(cst::StructDef),
    Enum(cst::EnumDef),
    // Not sure if states belong to the same namespace; AFAIK they're only ever referred to by name
    // (as in name literal) but needs verification.
}

/// # Conversion from CST
impl UntypedClassPartition {
    pub fn from_cst(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        file: cst::File,
    ) -> Self {
        let source = &sources.get(source_file_id).source;
        let class = file.class;

        let mut vars = IndexMap::new();
        let mut functions = IndexMap::new();
        let mut types = IndexMap::new();
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
                            Self::add_to_scope(
                                diagnostics,
                                sources,
                                source_file_id,
                                &mut types,
                                TypeCst::Struct(struct_def),
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
                        item_function,
                        |item_function| {
                            mangled_function_name(sources, source_file_id, item_function)
                                .into_owned()
                        },
                    );
                }
                cst::Item::Struct(item_struct) => {
                    Self::add_to_scope(
                        diagnostics,
                        sources,
                        source_file_id,
                        &mut types,
                        TypeCst::Struct(item_struct.def),
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
                        .with_note(notes::CPP_UNSUPPORTED),
                ),
                cst::Item::StructCppText(item_struct_cpp_text) => diagnostics.emit(
                    Diagnostic::error(source_file_id, "`structcpptext` may only appear in structs")
                        .with_label(Label::primary(item_struct_cpp_text.cpptext.span, "")),
                ),
                cst::Item::Stmt(stmt) => {
                    diagnostics.emit(
                        Diagnostic::error(source_file_id, "statement found outside of function")
                            .with_label(Label::primary(stmt.span(), "statements are not allowed here"))
                            .with_note(indoc!("
                                note: in contrast to most modern scripting languages, UnrealScript requires all executable code to belong
                                      to a function. this is because code is executed in response to game events such as `Tick`;
                                      it doesn't execute automatically like in Python or Lua
                            "))
                    );
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
            vars,
            functions,
            types,
            states,
            default_properties,
            replication,
        }
    }

    fn add_to_scope<I>(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        scope: &mut IndexMap<CaseInsensitive<String>, I>,
        item: I,
    ) where
        I: NamedItem,
    {
        Self::add_to_scope_with_name(diagnostics, sources, source_file_id, scope, item, |item| {
            sources.span(source_file_id, &item.name()).to_owned()
        })
    }

    fn add_to_scope_with_name<I>(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
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
                Self::redeclaration_error(
                    sources,
                    source_file_id,
                    other.name().span,
                    source_file_id,
                    item.name().span,
                )
                .with_note(Note {
                    kind: NoteKind::Debug,
                    text: format!("mangled name of conflicting item: `{name}`"),
                    suggestion: None,
                }),
            );
        } else {
            scope.insert(CaseInsensitive::new(name.to_owned()), item);
        }
    }

    fn redeclaration_error(
        sources: &SourceFileSet,
        source_file_id_first: SourceFileId,
        span_first: Span,
        source_file_id_re: SourceFileId,
        span_re: Span,
    ) -> Diagnostic {
        let source_re = &sources.get(source_file_id_re).source;
        let source_first = &sources.get(source_file_id_first).source;
        let name_re = span_re.get_input(source_re);
        let name_first = span_first.get_input(source_first);

        let mut diagnostic =
            Diagnostic::error(source_file_id_re, format!("redefinition of `{name_first}`"))
                .with_label(
                    Label::primary(span_first, "first defined here").in_file(source_file_id_first),
                )
                .with_label(Label::primary(span_re, "redefined here").in_file(source_file_id_re));

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
            generic: None,
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
    fn name(&self) -> token::Ident {
        self.variable.name
    }
}

impl NamedItem for VarCst {
    fn name(&self) -> token::Ident {
        match self {
            VarCst::Const(item_const) => item_const.name(),
            VarCst::Var(item_var) => item_var.name(),
        }
    }
}

impl NamedItem for TypeCst {
    fn name(&self) -> token::Ident {
        match self {
            TypeCst::Struct(item_struct) => item_struct.name(),
            TypeCst::Enum(item_enum) => item_enum.name(),
        }
    }
}

pub trait UntypedClassPartitionsExt {
    fn find_var(&self, name: &str) -> Option<(SourceFileId, &VarCst)>;
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
}
