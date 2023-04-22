use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{diagnostics::notes, ir::Sink, Compiler};

use super::{
    builder::FunctionBuilder,
    expr::{ExpectedType, ExprContext},
};

mod cond;
mod ifs;
mod local;
mod loops;
mod ret;

impl<'a> Compiler<'a> {
    pub fn stmt(&mut self, builder: &mut FunctionBuilder, stmt: &cst::Stmt) {
        match stmt {
            cst::Stmt::Empty(semi) => self.env.emit(
                Diagnostic::warning(builder.source_file_id, "empty statement has no effect")
                    .with_label(Label::primary(semi.span, "this semicolon is redundant")),
            ),
            cst::Stmt::Expr(expr) => self.stmt_expr(builder, expr),
            cst::Stmt::Block(block) => self.stmt_block(builder, block),

            cst::Stmt::Local(local) => self.stmt_local(builder, local),

            cst::Stmt::If(stmt) => self.stmt_if(builder, stmt),
            cst::Stmt::While(stmt) => self.stmt_while(builder, stmt),
            cst::Stmt::For(stmt) => self.stmt_for(builder, stmt),
            cst::Stmt::Return(ret) => self.stmt_return(builder, ret),

            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "unsupported statement")
                        .with_label(Label::primary(stmt.span(), ""))
                        .with_note(notes::WIP),
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
        builder.push_local_scope();
        for stmt in &block.stmts {
            self.stmt(builder, stmt);
        }
        builder.pop_local_scope();
    }
}
