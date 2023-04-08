use muscript_syntax::{
    cst,
    lexis::token::{Token, TokenKind},
};

use crate::{function::builder::FunctionBuilder, ir::RegisterId, Compiler};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_prefix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: Token,
        right: &cst::Expr,
    ) -> RegisterId {
        // TODO: Specialize negation here so that it's interpreted as if it were a part of
        // the literal.
        todo!()
    }
}
