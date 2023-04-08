use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};
use muscript_syntax::{cst, lexis::token::Token};

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
    pub(super) fn expr_prefix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        operator: Token,
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
        let right_ty = builder.ir.register(right).ty;
        let right_type_name = self.env.type_name(right_ty);
        let operator_str = self.sources.span(builder.source_file_id, &operator);
        let operator_function_name = mangled_operator_function_name(Operator::Prefix {
            operator: operator_str,
            parameter_type: right_type_name,
        });
        if let Some(function_id) = self.lookup_function(builder.class_id, &operator_function_name) {
            builder.ir.append_register(
                operator.span,
                "prefix",
                self.env.get_function(function_id).return_ty,
                Value::CallFinal {
                    function: function_id,
                    args: vec![right],
                },
            )
        } else {
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    format!(
                        "no overload of prefix operator `{}` exists for values of type `{}`",
                        operator_str,
                        self.env.type_name(right_ty)
                    ),
                )
                .with_label(Label::primary(operator.span, "")),
            );
            builder.ir.append_register(
                operator.span,
                "prefix_invalid",
                context.expected_type.to_type_id(),
                Value::Void,
            )
        }
    }
}
