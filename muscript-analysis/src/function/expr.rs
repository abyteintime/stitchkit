use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

use super::builder::FunctionBuilder;

mod call;
mod conversion;
mod lit;

#[derive(Debug, Clone)]
pub struct ExprContext {
    pub expected_type: ExpectedType,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedType {
    Matching(TypeId),
    Any,
}

impl ExpectedType {
    fn to_type_id(self) -> TypeId {
        match self {
            ExpectedType::Matching(type_id) => type_id,
            ExpectedType::Any => TypeId::VOID,
        }
    }

    fn is_exactly(&self, id: TypeId) -> bool {
        match self {
            ExpectedType::Matching(type_id) => *type_id == id,
            ExpectedType::Any => false,
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn expr(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        expr: &cst::Expr,
    ) -> RegisterId {
        match expr {
            cst::Expr::Lit(lit) => self.expr_lit(builder, context, lit),
            cst::Expr::Prefix { operator, right } => {
                self.expr_prefix(builder, context, operator, right)
            }
            cst::Expr::Postfix { left, operator } => {
                self.expr_postfix(builder, context, operator, left)
            }
            cst::Expr::Infix {
                left,
                operator,
                right,
            } => self.expr_infix(builder, context, operator, left, right),
            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id(), "unsupported expression")
                        .with_label(Label::primary(expr.span(), ""))
                        .with_note("note: MuScript is still unfinished; you can help contribute at <https://github.com/abyteintime/stitchkit>")
                );
                builder.ir.append_register(
                    expr.span(),
                    "unsupported",
                    context.expected_type.to_type_id(),
                    Value::Void,
                )
            }
        }
    }
}
