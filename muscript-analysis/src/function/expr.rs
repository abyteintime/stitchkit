use indoc::indoc;
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

mod array;
mod assign;
mod call;
mod conversion;
mod dot;
mod ident;
mod lit;
mod object;
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
            cst::Expr::Paren { inner, .. } => self.expr(builder, context, inner),

            cst::Expr::Lit(lit) => self.expr_lit(builder, context, lit),
            cst::Expr::Ident(ident) => self.expr_ident(builder, context, *ident),
            cst::Expr::Object { class, name } => {
                self.expr_object(builder, context, expr, *class, *name)
            }

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
            cst::Expr::Call {
                function,
                open,
                args,
                close,
                ..
            } => self.expr_call(
                builder,
                context,
                expr,
                call::CallSyntax {
                    function,
                    open: *open,
                    args,
                    close: *close,
                },
            ),

            cst::Expr::Dot { left, dot, field } => self.expr_dot(builder, expr, left, *dot, *field),
            cst::Expr::Index {
                left,
                open,
                index,
                close,
            } => self.expr_index(builder, expr, left, *open, index, *close),

            cst::Expr::Assign { lvalue, rvalue, .. } => {
                self.expr_assign(builder, context, lvalue, rvalue)
            }

            cst::Expr::FailedExp(token) => {
                let macro_name = self.sources.span(builder.source_file_id, &token.span);
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "use of undefined macro as an expression")
                        .with_label(Label::primary(expr.span(), ""))
                        .with_note(format!("the macro `{macro_name}` was not defined anywhere, and expanded to no tokens where an expression was expected"))

                        .with_note(format!(indoc!{"
                            help: try defining the macro somewhere:
                                  `define {} (2 + 2) // or something else
                        "}, macro_name)),
                );
                builder.ir.append_register(
                    expr.span(),
                    "failed_expansion",
                    TypeId::ERROR,
                    Value::Void,
                )
            }

            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "unsupported expression")
                        .with_label(Label::primary(expr.span(), ""))
                        .with_note(notes::WIP),
                );
                builder
                    .ir
                    .append_register(expr.span(), "unsupported", TypeId::ERROR, Value::Void)
            }
        }
    }
}
