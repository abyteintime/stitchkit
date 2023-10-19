use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};

use crate::{function::builder::FunctionBuilder, ir::RegisterId, Compiler, TypeId};

impl<'a> Compiler<'a> {
    pub(super) fn ensure_cond_is_bool(
        &mut self,
        builder: &FunctionBuilder,
        register_id: RegisterId,
    ) {
        let register = builder.ir.register(register_id);
        if register.ty != TypeId::BOOL && register.ty != TypeId::ERROR {
            // TODO: For various types, this could suggest fixes.
            // - MuScript doesn't allow ints to be used as `if` conditions, but could suggest to do
            //   `x != 0`.
            // - Same for objects, but could suggest to do `x != none`.
            self.env.emit(
                Diagnostic::error(format!(
                    "condition must be a `Bool`, but was found to be `{}`",
                    self.env.type_name(register.ty)
                ))
                .with_label(Label::primary(builder.ir.node(register_id.into()), "")),
            );
        }
    }
}
