use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, ReplacementSuggestion},
    span::Spanned,
};
use muscript_syntax::cst;

use crate::{
    function::{
        builder::FunctionBuilder,
        expr::{ExpectedType, ExprContext},
    },
    ir::{Terminator, Value},
    Compiler, TypeId,
};

impl<'a> Compiler<'a> {
    pub(super) fn stmt_return(&mut self, builder: &mut FunctionBuilder, ret: &cst::StmtReturn) {
        let return_value = match &ret.value {
            cst::ReturnValue::Nothing(semi) => {
                builder
                    .ir
                    .append_register(semi.span(), "return_void", TypeId::VOID, Value::Void)
            }
            cst::ReturnValue::Something(expr, _) => self.expr(
                builder,
                ExprContext {
                    expected_type: ExpectedType::Matching(builder.return_ty),
                },
                expr,
            ),
        };
        let value_ty = builder.ir.register(return_value).ty;
        let return_value = if self.check_return_value_presence(builder, ret, value_ty) {
            self.coerce_expr(builder, return_value, builder.return_ty)
        } else {
            return_value
        };

        builder.ir.set_terminator(Terminator::Return(return_value));
        let _unreachable = builder
            .ir
            .append_basic_block("unreachable_after_return", ret.kreturn.span());
    }

    /// Returns `true` if the return value's presence matches the return type.
    fn check_return_value_presence(
        &mut self,
        builder: &mut FunctionBuilder,
        ret: &cst::StmtReturn,
        provided_return_value_ty: TypeId,
    ) -> bool {
        if provided_return_value_ty == TypeId::ERROR {
            false
        } else if builder.return_ty == TypeId::VOID && provided_return_value_ty != TypeId::VOID {
            self.env.emit(
                Diagnostic::error(
                    "function does not return anything, but a return value was provided",
                )
                .with_label(Label::primary(&ret.value, ""))
                .with_label(Label::secondary(
                    &builder.function(self.env).name,
                    "function declared here",
                ))
                .with_note((
                    "help: try removing the return value",
                    self.sources.replacement_suggestion(ret, "return;"),
                )),
            );
            false
        } else if builder.return_ty != TypeId::VOID && provided_return_value_ty == TypeId::VOID {
            self.env.emit(
                Diagnostic::error(format!(
                    "function was declared to return `{}`, but no return value was provided",
                    self.env.type_name(builder.return_ty)
                ))
                .with_label(Label::primary(ret, ""))
                .with_label(Label::secondary(
                    &builder.function(self.env).name,
                    "function declared here",
                ))
                .with_note((
                    "help: try adding a return value",
                    // TODO: Type-specific suggestions?
                    self.sources
                        .replacement_suggestion(ret, "return SomeValueHere;"),
                )),
            );
            false
        } else {
            true
        }
    }
}
