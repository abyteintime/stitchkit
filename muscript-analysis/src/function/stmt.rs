use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{ir::Sink, Compiler};

use super::{
    builder::FunctionBuilder,
    expr::{ExpectedType, ExprContext},
};

mod ret;

impl<'a> Compiler<'a> {
    pub fn stmt(&mut self, builder: &mut FunctionBuilder, stmt: &cst::Stmt) {
        match stmt {
            cst::Stmt::Empty(semi) => self.env.emit(
                Diagnostic::warning(builder.source_file_id(), "empty statement has no effect")
                    .with_label(Label::primary(semi.span, "this semicolon is redundant")),
            ),
            cst::Stmt::Expr(expr) => self.stmt_expr(builder, expr),
            cst::Stmt::Block(block) => self.stmt_block(builder, block),
            cst::Stmt::Return(ret) => self.stmt_return(builder, ret),
            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id(), "unsupported statement")
                        .with_label(Label::primary(stmt.span(), ""))
                        .with_note("note: MuScript is still unfinished; you can help contribute at <https://github.com/abyteintime/stitchkit>")
                );
            }
        }
    }

    fn stmt_expr(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtExpr) {
        let register_id = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            &stmt.expr,
        );
        builder
            .ir
            .append_sink(stmt.span(), Sink::Discard(register_id));
    }

    pub(crate) fn stmt_block(&mut self, builder: &mut FunctionBuilder, block: &cst::Block) {
        // TODO: Scope.
        for stmt in &block.stmts {
            self.stmt(builder, stmt);
        }
    }
}
