use indexmap::IndexMap;
use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    source::{SourceFileId, SourceFileSet, Span, Spanned},
};
use muscript_syntax::{cst, lexis::token};

use crate::diagnostics::{self, notes, unnecessary_semicolon};

use super::{InlineTypeDef, ItemSingleVar, TypeCst, UntypedClassPartition};

#[derive(Debug, Clone)]
pub struct UntypedStruct {
    pub name: token::Ident,
    pub extends: Option<cst::Path>,

    pub vars: IndexMap<CaseInsensitive<String>, ItemSingleVar>,

    // structdefaultproperties is normalized to regular defaultproperties, because requiring the
    // extra `struct` word is just silly.
    pub default_properties: Option<cst::ItemDefaultProperties>,
}

/// # Conversion from CST
impl UntypedStruct {
    pub fn from_cst(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        types: &mut IndexMap<CaseInsensitive<String>, TypeCst>,
        cst: cst::StructDef,
    ) -> Self {
        let mut vars = IndexMap::new();
        let mut default_properties = None;

        for item in cst.items {
            match item {
                cst::Item::Empty(semi) => {
                    diagnostics.emit(unnecessary_semicolon(source_file_id, semi).with_note(
                        indoc! {"
                            note: each `var` declaration needs a single semicolon after it;
                                  having one anywhere else is redundant
                        "},
                    ));
                }
                cst::Item::Var(mut item_var) => {
                    match UntypedClassPartition::lower_inline_type_def(&mut item_var.ty) {
                        Some(InlineTypeDef::Enum(enum_def)) => {
                            UntypedClassPartition::add_to_scope(
                                diagnostics,
                                sources,
                                source_file_id,
                                types,
                                TypeCst::Enum(enum_def),
                            );
                        }
                        Some(InlineTypeDef::Struct(struct_def)) => {
                            let untyped_struct = UntypedStruct::from_cst(
                                diagnostics,
                                sources,
                                source_file_id,
                                types,
                                struct_def,
                            );
                            UntypedClassPartition::add_to_scope(
                                diagnostics,
                                sources,
                                source_file_id,
                                types,
                                TypeCst::Struct(untyped_struct),
                            );
                        }
                        None => (),
                    }
                    for var in ItemSingleVar::lower(item_var) {
                        UntypedClassPartition::add_to_scope(
                            diagnostics,
                            sources,
                            source_file_id,
                            &mut vars,
                            var,
                        );
                    }
                }

                cst::Item::DefaultProperties(cst::ItemDefaultProperties {
                    keyword: cst::KDefaultProperties { span },
                    block,
                })
                | cst::Item::StructDefaultProperties(cst::ItemStructDefaultProperties {
                    keyword: cst::KStructDefaultProperties { span },
                    block,
                }) => {
                    // As mentioned, canonicalize `structdefaultproperties` items to
                    // regular `defaultproperties`. There's no reason to have this distinction
                    // anyways.
                    default_properties = Some(cst::ItemDefaultProperties {
                        keyword: cst::KDefaultProperties { span },
                        block,
                    })
                }

                cst::Item::Const(item_const) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_const.span(),
                        "`const` may not appear in structs",
                    )
                    .with_note("help: try putting your `const` outside the struct"),
                ),
                cst::Item::Simulated(item_simulated) => match item_simulated.item {
                    cst::SimulatedItem::Function(item_function) => diagnostics.emit(
                        item_may_not_appear_in_struct(
                            source_file_id,
                            item_function.span(),
                            "functions may not appear in structs",
                        )
                        .with_note("help: try putting your function outside the struct"),
                    ),
                    cst::SimulatedItem::State(item_state) => diagnostics.emit(
                        item_may_not_appear_in_struct(
                            source_file_id,
                            item_state.span(),
                            "states may not appear in structs",
                        )
                        .with_note("help: try putting your state outside the struct"),
                    ),
                },
                cst::Item::Function(item_function) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_function.span(),
                        "functions may not appear in structs",
                    )
                    .with_note("help: try putting your function outside the struct"),
                ),
                cst::Item::Struct(item_struct) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_struct.span(),
                        "structs may not nest",
                    )
                    .with_label(Label::secondary(cst.open.span, "outer struct begins here"))
                    .with_label(Label::secondary(cst.close.span, "outer struct ends here"))
                    .with_note("help: try putting your struct outside this struct's braces"),
                ),
                cst::Item::Enum(item_enum) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_enum.span(),
                        "enums may not appear in structs",
                    )
                    .with_note("help: try putting your enum outside the struct"),
                ),
                cst::Item::State(item_state) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_state.span(),
                        "states may not appear in structs",
                    )
                    .with_note("help: try putting your state outside the struct"),
                ),
                cst::Item::Replication(item_replication) => diagnostics.emit(
                    item_may_not_appear_in_struct(
                        source_file_id,
                        item_replication.span(),
                        "replication blocks may not appear in structs",
                    )
                    .with_note(indoc! {"
                        note: replication conditions can only be specified for class variables;
                              structs are plain data and do not encapsulate behavior,
                              therefore they always replicate as a whole
                    "}),
                ),
                item @ (cst::Item::CppText(_) | cst::Item::StructCppText(_)) => diagnostics.emit(
                    Diagnostic::warning(source_file_id, "`cpptext` item is ignored")
                        .with_label(Label::primary(item.span(), ""))
                        .with_note(notes::CPP_UNSUPPORTED),
                ),
                cst::Item::Stmt(stmt) => diagnostics.emit(diagnostics::stmt_outside_of_function(
                    source_file_id,
                    stmt.span(),
                )),
            }
        }

        Self {
            name: cst.name,
            extends: cst.extends.map(|x| x.parent_class),
            vars,
            default_properties,
        }
    }
}

fn item_may_not_appear_in_struct(
    source_file_id: SourceFileId,
    span: Span,
    message: &str,
) -> Diagnostic {
    Diagnostic::error(source_file_id, message)
        .with_label(Label::primary(span, ""))
        .with_note("note: structs may only contain `var`s and `defaultproperties`")
}
