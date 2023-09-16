use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Sink},
    Compiler, TypeId,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    pub(super) fn expr_assign(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        lvalue: &cst::Expr,
        rvalue: &cst::Expr,
    ) -> RegisterId {
        let lvalue_register = self.expr(builder, context, lvalue);
        if builder.ir.register(lvalue_register).ty != TypeId::ERROR
            && !builder.ir.is_place(lvalue_register)
        {
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    "left-hand side of `=` is not a place that can be assigned to",
                )
                .with_label(Label::primary(
                    lvalue.span(),
                    "this is not a place in memory",
                )),
            )
        }

        let rvalue_register = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Matching(builder.ir.register(lvalue_register).ty),
            },
            rvalue,
        );
        let rvalue_register = self.coerce_expr(
            builder,
            rvalue_register,
            builder.ir.register(lvalue_register).ty,
        );
        builder.ir.append_sink(
            lvalue.span().join(&rvalue.span()),
            Sink::Store(lvalue_register, rvalue_register),
        );
        // Return the lvalue register instead of the rvalue register, such that in case of register
        // reuse we get a register that can be considered cheap to reuse (so that an intermediary
        // variable is not needed).
        lvalue_register
    }
}
