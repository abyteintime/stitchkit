use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};
use muscript_syntax::lexis::token::Ident;

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    Compiler,
};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_ident(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        ident: Ident,
    ) -> RegisterId {
        let name = self.sources.span(builder.source_file_id, &ident);
        if let Some(var_id) = builder.lookup_local(name) {
            let ty = self.env.get_var(var_id).ty;
            builder
                .ir
                .append_register(ident.span, name.to_owned(), ty, Value::Local(var_id))
        } else if let Some(var_id) = self.lookup_class_var(builder.class_id, name) {
            let ty = self.env.get_var(var_id).ty;
            builder
                .ir
                .append_register(ident.span, name.to_owned(), ty, Value::Field(var_id))
        } else {
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    format!("cannot find variable `{name}` in this scope"),
                )
                .with_label(Label::primary(ident.span, "")),
            );
            builder.ir.append_register(
                ident.span,
                "unknown_ident",
                context.expected_type.to_type_id(),
                Value::Void,
            )
        }
    }
}
