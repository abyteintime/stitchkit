use std::fmt::Write as _;

use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::{
    cst::{self, InfixOperator},
    lexis::token::Token,
};

use crate::{
    function::{
        builder::FunctionBuilder,
        mangling::{mangled_operator_function_name, Operator},
    },
    ir::{RegisterId, Value},
    Compiler,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    fn expr_operator(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: &dyn Spanned,
        is_prefix: bool,
        arguments: &[RegisterId],
    ) -> RegisterId {
        let operator_str = operator
            .span()
            .get_input(self.sources.source(builder.source_file_id));
        let operator_function_name = mangled_operator_function_name(Operator {
            operator: operator_str,
            argument_types: arguments
                .iter()
                .map(|&register_id| self.env.type_name(builder.ir.register(register_id).ty)),
            is_prefix,
        });
        if let Some(function_id) = self.lookup_function(builder.class_id, &operator_function_name) {
            builder.ir.append_register(
                operator.span(),
                "op",
                self.env.get_function(function_id).return_ty,
                Value::CallFinal {
                    function: function_id,
                    arguments: arguments.to_owned(),
                },
            )
        } else {
            let mut error = format!(
                "no overload of {} `{}` exists for {} of type ",
                if is_prefix {
                    "prefix operator"
                } else {
                    "operator"
                },
                operator_str,
                if arguments.len() > 1 {
                    "arguments"
                } else {
                    "argument"
                },
            );
            for (i, &register_id) in arguments.iter().enumerate() {
                if i != 0 {
                    error.push_str(", ");
                }
                let type_name = self.env.type_name(builder.ir.register(register_id).ty);
                _ = write!(error, "`{type_name}`");
            }
            self.env.emit(
                Diagnostic::error(builder.source_file_id, error)
                    .with_label(Label::primary(operator.span(), "")),
            );
            builder.ir.append_register(
                operator.span(),
                "op",
                context.expected_type.to_type_id(),
                Value::Void,
            )
        }
    }

    pub(super) fn expr_prefix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: &Token,
        right: &cst::Expr,
    ) -> RegisterId {
        // TODO: Specialize negation here so that it's interpreted as if it were a part of
        // the literal. Either that, or constant folding maybe?
        let right = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            right,
        );
        self.expr_operator(builder, context, operator, true, &[right])
    }

    pub(super) fn expr_postfix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: &Token,
        left: &cst::Expr,
    ) -> RegisterId {
        let left = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        self.expr_operator(builder, context, operator, false, &[left])
    }

    pub(super) fn expr_infix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: &InfixOperator,
        left: &cst::Expr,
        right: &cst::Expr,
    ) -> RegisterId {
        let left = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        let right = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            right,
        );
        self.expr_operator(builder, context, operator, false, &[left, right])
    }
}
