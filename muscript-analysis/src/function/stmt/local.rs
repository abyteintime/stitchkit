use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    class::{Var, VarFlags, VarKind},
    diagnostics::notes,
    function::builder::FunctionBuilder,
    Compiler, VarId,
};

impl<'a> Compiler<'a> {
    pub(super) fn stmt_local(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtLocal) {
        let ty = self.type_id(builder.source_file_id, builder.class_id, &stmt.ty);
        for var_def in &stmt.vars {
            if let Some(cpptype) = &var_def.cpptype {
                self.env.emit(
                    Diagnostic::error(
                        builder.source_file_id,
                        "C++ type is not allowed on local variables",
                    )
                    .with_label(Label::primary(cpptype.span(), "")),
                );
            }
            if let Some(meta) = &var_def.meta {
                self.env.emit(
                    Diagnostic::error(
                        builder.source_file_id,
                        "metadata are not allowed on local variables",
                    )
                    .with_label(Label::primary(meta.span(), "")),
                );
            }

            if let Some(array) = &var_def.array {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "arrays are not supported yet")
                        .with_label(Label::primary(array.span(), ""))
                        .with_note(notes::WIP),
                );
            }

            let var_id = self.env.register_var(Var {
                source_file_id: builder.source_file_id,
                name: var_def.name,
                ty,
                kind: VarKind::Var(VarFlags::empty()),
            });
            builder.ir.add_local(var_id);
            self.declare_local(builder, var_id);
        }
    }

    pub(crate) fn declare_local(&mut self, builder: &mut FunctionBuilder, var_id: VarId) {
        let var = self.env.get_var(var_id);
        let name_str = self.sources.span(builder.source_file_id, &var.name);
        if let Some(var_id) = builder.add_local_to_scope(name_str, var_id) {
            let previous_var = self.env.get_var(var_id);
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    format!("redefinition of variable `{name_str}`"),
                )
                .with_label(Label::primary(var.name.span, ""))
                .with_label(Label::secondary(
                    previous_var.name.span,
                    "previous definition here",
                )),
            )
        }
    }
}
