use muscript_syntax::cst;

use crate::{
    function::{
        builder::FunctionBuilder,
        expr::{ExpectedType, ExprContext},
    },
    ir::{Sink, Terminator},
    Compiler, TypeId,
};

impl<'a> Compiler<'a> {
    pub(super) fn stmt_while(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtWhile) {
        let before_cond = builder.ir.cursor();

        let while_cond_begin = builder.ir.append_basic_block("while_cond");
        let condition = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Matching(TypeId::BOOL),
            },
            &stmt.cond.expr,
        );
        self.ensure_cond_is_bool(builder, condition);
        let while_cond_end = builder.ir.cursor();

        let while_body_begin = builder.ir.append_basic_block("while_body");
        self.stmt(builder, &stmt.body);
        builder
            .ir
            .set_terminator(Terminator::Goto(while_cond_begin));

        let past_while = builder.ir.append_basic_block("past_while");

        builder.ir.set_cursor(before_cond);
        builder
            .ir
            .set_terminator(Terminator::Goto(while_cond_begin));

        builder.ir.set_cursor(while_cond_end);
        builder.ir.set_terminator(Terminator::GotoIf {
            condition,
            if_true: while_body_begin,
            if_false: past_while,
        });

        builder.ir.set_cursor(past_while);
    }

    pub(super) fn stmt_for(&mut self, builder: &mut FunctionBuilder, stmt: &cst::StmtFor) {
        let init = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            &stmt.init,
        );
        let init_span = builder.ir.node(init.into()).span;
        builder.ir.append_sink(init_span, Sink::Discard(init));

        // This is almost exactly the same as `stmt_while`, just with the extra update expression.

        let before_cond = builder.ir.cursor();

        let for_cond_begin = builder.ir.append_basic_block("for_cond");
        let condition = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            &stmt.cond,
        );
        self.ensure_cond_is_bool(builder, condition);
        let for_cond_end = builder.ir.cursor();

        let for_body_begin = builder.ir.append_basic_block("for_body");
        self.stmt(builder, &stmt.body);
        let for_body_end = builder.ir.cursor();

        // TODO: Determine what happens with `continue`s. Perhaps the update expression should be
        // inlined instead of jumped to?
        let for_update_begin = builder.ir.append_basic_block("for_update");
        let update = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            &stmt.update,
        );
        let update_span = builder.ir.node(update.into()).span;
        builder.ir.append_sink(update_span, Sink::Discard(update));
        builder.ir.set_terminator(Terminator::Goto(for_cond_begin));

        let past_for = builder.ir.append_basic_block("past_for");

        builder.ir.set_cursor(before_cond);
        builder.ir.set_terminator(Terminator::Goto(for_cond_begin));

        builder.ir.set_cursor(for_cond_end);
        builder.ir.set_terminator(Terminator::GotoIf {
            condition,
            if_true: for_body_begin,
            if_false: past_for,
        });

        builder.ir.set_cursor(for_body_end);
        builder
            .ir
            .set_terminator(Terminator::Goto(for_update_begin));

        builder.ir.set_cursor(past_for);
    }
}
