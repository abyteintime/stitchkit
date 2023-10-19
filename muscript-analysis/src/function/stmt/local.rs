use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};
use muscript_syntax::cst::{self, ItemName};

use crate::{
    class::{Var, VarFlags, VarKind},
    diagnostics::notes,
    function::builder::FunctionBuilder,
    Compiler, VarId,
};

impl<'a> Compiler<'a> {
    pub(super) fn stmt_local(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtLocal) {
        let ty = self.type_id(builder.class_id, &stmt.ty);
        for var_def in &stmt.vars {
            if let Some(cpptype) = &var_def.cpptype {
                self.env.emit(
                    Diagnostic::error("C++ type is not allowed on local variables")
                        .with_label(Label::primary(cpptype, "")),
                );
            }
            if let Some(meta) = &var_def.meta {
                self.env.emit(
                    Diagnostic::error("metadata are not allowed on local variables")
                        .with_label(Label::primary(meta, "")),
                );
            }

            if let Some(array) = &var_def.array {
                self.env.emit(
                    Diagnostic::error("arrays are not supported yet")
                        .with_label(Label::primary(array, ""))
                        .with_note(notes::WIP),
                );
            }

            let var_id = self.env.register_var(Var {
                name: ItemName::from_spanned(&var_def.name),
                ty,
                kind: VarKind::Var(VarFlags::empty()),
            });
            builder.ir.add_local(var_id);
            self.declare_local(builder, var_id);
        }
    }

    pub(crate) fn declare_local(&mut self, builder: &mut FunctionBuilder, var_id: VarId) {
        let var = self.env.get_var(var_id);
        let name_str = self.sources.source(&var.name);
        if let Some(var_id) = builder.add_local_to_scope(name_str, var_id) {
            let previous_var = self.env.get_var(var_id);
            self.env.emit(
                Diagnostic::error(format!("redefinition of variable `{name_str}`"))
                    .with_label(Label::primary(&var.name, ""))
                    .with_label(Label::secondary(
                        &previous_var.name,
                        "previous definition here",
                    )),
            )
        }
    }
}
