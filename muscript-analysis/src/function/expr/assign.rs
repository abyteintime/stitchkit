use muscript_foundation::source::Spanned;
use muscript_syntax::cst;

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Sink},
    Compiler,
};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_assign(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        lvalue: &cst::Expr,
        rvalue: &cst::Expr,
    ) -> RegisterId {
        // Use the same context for the lvalue, since we know its type must be the same as
        // the rvalue's.
        let lvalue_register = self.expr(builder, context.clone(), lvalue);
        let rvalue_register = self.expr(builder, context, rvalue);
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
