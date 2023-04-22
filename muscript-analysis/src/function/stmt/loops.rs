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
}
