use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

impl<'a> Compiler<'a> {
    pub fn coerce_expr(
        &mut self,
        builder: &mut FunctionBuilder,
        input_register_id: RegisterId,
        expected_ty: TypeId,
    ) -> RegisterId {
        let input_node = builder.ir.node(input_register_id.into());
        let input_register = builder.ir.register(input_register_id);

        if !matches!(input_register.value, Value::Void)
            && expected_ty != TypeId::VOID
            && input_register.ty != expected_ty
        {
            self.env.emit(
                Diagnostic::error(builder.source_file_id, "type mismatch")
                    .with_label(Label::primary(input_node.span, ""))
                    .with_note(indoc::formatdoc! {"
                            expected `{}`
                                 got `{}`
                        ",
                        self.env.type_name(expected_ty),
                        self.env.type_name(input_register.ty)
                    }),
            )
        }

        input_register_id
    }
}
