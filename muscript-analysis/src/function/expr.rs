use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    diagnostics::notes,
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

use super::builder::FunctionBuilder;

mod assign;
mod call;
mod conversion;
mod ident;
mod lit;
mod void_handling;

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
            cst::Expr::Ident(ident) => self.expr_ident(builder, context, *ident),

            cst::Expr::Prefix { operator, right } => {
                self.expr_prefix(builder, context, expr, operator, right)
            }
            cst::Expr::Postfix { left, operator } => {
                self.expr_postfix(builder, context, expr, operator, left)
            }
            cst::Expr::Infix {
                left,
                operator,
                right,
            } => self.expr_infix(builder, context, expr, operator, left, right),
            cst::Expr::Paren {
                open: _,
                inner,
                close: _,
            } => self.expr(builder, context, inner),

            cst::Expr::Assign {
                lvalue,
                assign: _,
                rvalue,
            } => self.expr_assign(builder, context, lvalue, rvalue),

            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "unsupported expression")
                        .with_label(Label::primary(expr.span(), ""))
                        .with_note(notes::WIP),
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
