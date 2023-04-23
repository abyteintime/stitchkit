use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::{cst, lexis::token::Ident};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    type_system::Type,
    Compiler, TypeId,
};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_dot(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        left: &cst::Expr,
        field: Ident,
    ) -> RegisterId {
        let left_register_id = self.expr(builder, context, left);
        let left_type_id = builder.ir.register(left_register_id).ty;

        let field_name = self.sources.span(builder.source_file_id, &field);

        match self.env.get_type(left_type_id) {
            &Type::Object(class_id) => {
                if let Some(var_id) = self.lookup_class_var(class_id, field_name) {
                    let field_ty = self.env.get_var(var_id).ty;
                    let field = builder.ir.append_register(
                        field.span,
                        field_name.to_owned(),
                        field_ty,
                        Value::Field(var_id),
                    );
                    builder.ir.append_register(
                        outer.span(),
                        field_name.to_owned(),
                        field_ty,
                        Value::In {
                            context: left_register_id,
                            action: field,
                        },
                    )
                } else {
                    // TODO: If a function with the same name exists, suggest calling it.
                    self.env.emit(
                        Diagnostic::error(
                            builder.source_file_id,
                            format!(
                                "cannot find variable `{}` in class `{}`",
                                field_name,
                                self.env.class_name(class_id)
                            ),
                        )
                        .with_label(Label::primary(field.span, "")),
                    );
                    builder.ir.append_register(
                        outer.span(),
                        "invalid_field",
                        TypeId::VOID,
                        Value::Void,
                    )
                }
            }
            Type::Array(_) => {
                self.env.emit(
                    Diagnostic::bug(
                        builder.source_file_id,
                        "`.` on arrays is not yet implemented",
                    )
                    .with_label(Label::primary(field.span, "")),
                );
                builder
                    .ir
                    .append_register(outer.span(), "array_dot", TypeId::VOID, Value::Void)
            }
            Type::Struct { outer: _ } => {
                self.env.emit(
                    Diagnostic::bug(
                        builder.source_file_id,
                        "`.` on structs is not yet implemented",
                    )
                    .with_label(Label::primary(field.span, "")),
                );
                builder
                    .ir
                    .append_register(outer.span(), "struct_dot", TypeId::VOID, Value::Void)
            }
            _ => {
                self.env.emit(
                    Diagnostic::error(
                        builder.source_file_id,
                        "the `.` operator can only be used on objects, structs, and arrays",
                    )
                    .with_label(Label::primary(field.span, "")),
                );
                builder
                    .ir
                    .append_register(outer.span(), "invalid_dot", TypeId::VOID, Value::Void)
            }
        }
    }
}
