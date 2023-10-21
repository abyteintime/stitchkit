use muscript_foundation::span::Spanned;
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
        self.ensure_cond_is_bool(builder, condition);
        let at_cond_block = builder.ir.cursor();

        // NOTE: There may occur more basic blocks inside the branch, so we need to probe for its
        // ending position and place an unconditional goto terminator _there_ instead of the block
        // we just create. The same goes for the false branch.
        let if_true_begin = builder
            .ir
            .append_basic_block("if_true", stmt.true_branch.span());
        self.stmt(builder, &stmt.true_branch);
        let if_true_end = builder.ir.cursor();

        let if_false = stmt.false_branch.as_ref().map(|false_branch| {
            let if_false_begin = builder
                .ir
                .append_basic_block("if_false", stmt.false_branch.span());
            self.stmt(builder, &false_branch.then);
            (if_false_begin, builder.ir.cursor())
        });

        let past_if = builder.ir.append_basic_block("past_if", stmt.span());

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
