use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    span::Spanned,
};
use muscript_syntax::lexis::token::Ident;

use crate::{
    class::VarKind,
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
        let name = self.sources.source(&ident);
        if let Some(var_id) = builder.lookup_local(name) {
            let ty = self.env.get_var(var_id).ty;
            builder
                .ir
                .append_register(ident.span(), name.to_owned(), ty, Value::Local(var_id))
        } else if let Some(var_id) = self.lookup_class_var(builder.class_id, name) {
            let var = self.env.get_var(var_id);
            let ty = var.ty;
            match &var.kind {
                VarKind::Var(_) => builder.ir.append_register(
                    ident.span(),
                    name.to_owned(),
                    ty,
                    Value::Field(var_id),
                ),
                VarKind::Const(constant) => {
                    constant.append_to(&mut builder.ir, ident.span(), "const")
                }
            }
        } else if name.eq_ignore_ascii_case("self") {
            let ty = self.env.class_type(builder.class_id);
            builder
                .ir
                .append_register(ident.span(), "self", ty, Value::This)
        } else {
            self.env.emit(
                Diagnostic::error(format!("cannot find variable `{name}` in this scope"))
                    .with_label(Label::primary(&ident, "")),
            );
            builder.ir.append_register(
                ident.span(),
                "unknown_ident",
                context.expected_type.to_type_id(),
                Value::Void,
            )
        }
    }
}
