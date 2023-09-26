use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    span::Spanned,
};
use muscript_syntax::{
    cst,
    lexis::token::{Dot, Ident},
};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    type_system::Type,
    ClassId, Compiler, TypeId,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    pub(super) fn expr_dot(
        &mut self,
        builder: &mut FunctionBuilder,
        outer: &cst::Expr,
        left: &cst::Expr,
        dot: Dot,
        field: Ident,
    ) -> RegisterId {
        let left_register_id = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        let left_type_id = builder.ir.register(left_register_id).ty;

        let field_name = self.sources.source(&field);

        match self.env.get_type(left_type_id) {
            &Type::Object(class_id) => self.expr_dot_on_object(
                class_id,
                field_name,
                builder,
                field,
                outer,
                left_register_id,
            ),

            Type::Array(_) => {
                self.expr_dot_on_array(field_name, builder, field, outer, left_register_id)
            }

            &Type::Struct {
                outer: struct_outer_class,
            } => self.expr_dot_on_struct(
                struct_outer_class,
                field_name,
                builder,
                field,
                outer,
                left_register_id,
            ),

            _ => {
                // TODO: Also classes to get defaults and constants.
                if left_type_id != TypeId::ERROR {
                    self.env.emit(
                        Diagnostic::error(
                            "the `.` operator can only be used on objects, structs, and arrays",
                        )
                        .with_label(Label::primary(&dot, ""))
                        .with_label(Label::secondary(
                            left,
                            format!(
                                "this is found to be of type `{}`, which does not have fields",
                                self.env.type_name(left_type_id)
                            ),
                        )),
                    );
                }
                builder
                    .ir
                    .append_register(outer.span(), "invalid_dot", TypeId::ERROR, Value::Void)
            }
        }
    }

    fn expr_dot_on_object(
        &mut self,
        class_id: ClassId,
        field_name: &str,
        builder: &mut FunctionBuilder,
        field: Ident,
        outer: &cst::Expr,
        left_register_id: RegisterId,
    ) -> RegisterId {
        if let Some(var_id) = self.lookup_class_var(class_id, field_name) {
            let field_ty = self.env.get_var(var_id).ty;
            let field = builder.ir.append_register(
                field.span(),
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
                Diagnostic::error(format!(
                    "cannot find variable `{field_name}` in class `{}`",
                    self.env.class_name(class_id)
                ))
                .with_label(Label::primary(&field, "")),
            );
            builder
                .ir
                .append_register(outer.span(), "invalid_field", TypeId::ERROR, Value::Void)
        }
    }

    fn expr_dot_on_struct(
        &mut self,
        struct_outer_class: ClassId,
        field_name: &str,
        builder: &mut FunctionBuilder,
        field: Ident,
        outer: &cst::Expr,
        left_register_id: RegisterId,
    ) -> RegisterId {
        let struct_type = builder.ir.register(left_register_id).ty;
        // Drop generic arguments if any, as they are not used for lookup.
        let struct_type_name = self.env.type_name(struct_type).name.clone();

        if let Some(var_id) =
            self.lookup_struct_var(struct_outer_class, &struct_type_name, field_name)
        {
            let field_ty = self.env.get_var(var_id).ty;
            let field = builder.ir.append_register(
                field.span(),
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
            self.env.emit(
                Diagnostic::bug(format!(
                    "cannot find variable `{field_name}` in struct `{}`",
                    self.env.type_name(struct_type)
                ))
                .with_label(Label::primary(&field, "")),
            );
            builder
                .ir
                .append_register(outer.span(), "invalid_field", TypeId::ERROR, Value::Void)
        }
    }

    fn expr_dot_on_array(
        &mut self,
        field_name: &str,
        builder: &mut FunctionBuilder,
        field: Ident,
        outer: &cst::Expr,
        left_register_id: RegisterId,
    ) -> RegisterId {
        if CaseInsensitive::new_ref(field_name) == CaseInsensitive::new_ref("length") {
            builder.ir.append_register(
                outer.span(),
                "array_len",
                TypeId::INT,
                Value::Len(left_register_id),
            )
        } else {
            self.env.emit(
                Diagnostic::error("`Length` expected")
                    .with_label(Label::primary(&field, ""))
                    .with_note("note: arrays do not have properties other than `Length`"),
            );
            builder.ir.append_register(
                outer.span(),
                "array_invalid_field",
                TypeId::ERROR,
                Value::Void,
            )
        }
    }
}
