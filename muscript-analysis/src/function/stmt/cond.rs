use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    function::{
        builder::FunctionBuilder,
        expr::{ExpectedType, ExprContext},
    },
    ir::Terminator,
    Compiler, TypeId,
};

impl<'a> Compiler<'a> {
    pub(super) fn stmt_if(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtIf) {
        let condition = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Matching(TypeId::BOOL),
            },
            &stmt.cond.expr,
        );
        let cond_ty = builder.ir.register(condition).ty;
        if cond_ty != TypeId::BOOL {
            // TODO: For various types, this could suggest fixes.
            // - MuScript doesn't allow ints to be used as `if` conditions, but could suggest to do
            //   `x != 0`.
            // - Same for objects, but could suggest to do `x != none`.
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    format!(
                        "`if` condition must be a `Bool`, but was found to be `{}`",
                        self.env.type_name(cond_ty)
                    ),
                )
                .with_label(Label::primary(stmt.cond.expr.span(), "")),
            );
        }
        let at_cond_block = builder.ir.cursor();

        // NOTE: There may occur more basic blocks inside the branch, so we need to probe for its
        // ending position and place an unconditional goto terminator _there_ instead of the block
        // we just create. The same goes for the false branch.
        let if_true_begin = builder.ir.append_basic_block("if_true");
        self.stmt(builder, &stmt.true_branch);
        let if_true_end = builder.ir.cursor();

        let if_false = stmt.false_branch.as_ref().map(|false_branch| {
            let if_false_begin = builder.ir.append_basic_block("if_false");
            self.stmt(builder, &false_branch.then);
            (if_false_begin, builder.ir.cursor())
        });

        let past_if = builder.ir.append_basic_block("past_if");

        builder.ir.set_cursor(at_cond_block);
        builder.ir.set_terminator(Terminator::GotoIf {
            condition,
            if_true: if_true_begin,
            if_false: if_false.map(|(begin, _end)| begin).unwrap_or(past_if),
        });

        builder.ir.set_cursor(if_true_end);
        builder.ir.set_terminator(Terminator::Goto(past_if));
        if let Some((_begin, end)) = if_false {
            builder.ir.set_cursor(end);
            builder.ir.set_terminator(Terminator::Goto(past_if));
        }

        builder.ir.set_cursor(past_if);
    }
}
