use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    span::Spanned,
};
use muscript_syntax::{cst, token};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    type_system::Type,
    Compiler, TypeId,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    pub(super) fn expr_index(
        &mut self,
        builder: &mut FunctionBuilder,
        outer: &cst::Expr,
        left: &cst::Expr,
        open: token::LeftBracket,
        index: &cst::Expr,
        close: token::RightBracket,
    ) -> RegisterId {
        let left_register_id = self.expr(
            builder,
            // TODO: Could probably augment this somehow with a hint that this type should be an
            // array. Not sure how much that would improve inference though.
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        let left_type_id = builder.ir.register(left_register_id).ty;
        if let &Type::Array(element_type_id) = self.env.get_type(left_type_id) {
            let index_register_id = self.expr(
                builder,
                ExprContext {
                    expected_type: ExpectedType::Matching(TypeId::INT),
                },
                index,
            );
            let index_type_id = builder.ir.register(index_register_id).ty;
            if index_type_id != TypeId::INT {
                self.env.emit(
                    Diagnostic::error("array index must be an `Int`").with_label(Label::primary(
                        left,
                        format!(
                            "this was found to be of type `{}`",
                            self.env.type_name(index_type_id)
                        ),
                    )),
                );
            }
            builder.ir.append_register(
                outer.span(),
                "array_index",
                element_type_id,
                Value::Index {
                    array: left_register_id,
                    index: index_register_id,
                },
            )
        } else {
            self.env.emit(
                Diagnostic::error("indexing `[]` can only be done on arrays")
                    .with_label(Label::primary(
                        left,
                        format!(
                            "found to be of type `{}`, but an array was expected",
                            self.env.type_name(left_type_id)
                        ),
                    ))
                    .with_label(Label::secondary(
                        &open.span().join(&close.span()),
                        "indexing here",
                    )),
            );
            builder
                .ir
                .append_register(outer.span(), "non_array_index", TypeId::ERROR, Value::Void)
        }
    }
}
