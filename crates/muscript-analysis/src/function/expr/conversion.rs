use indoc::formatdoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    span::Spanned,
};
use muscript_lexer::token::{Token, TokenSpan};
use muscript_syntax::cst;

use crate::{
    function::builder::FunctionBuilder,
    ir::{PrimitiveCast, RegisterId, Value},
    type_system::{Primitive, Type},
    ClassId, Compiler, TypeId,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    pub fn coerce_expr(
        &mut self,
        builder: &mut FunctionBuilder,
        input_register_id: RegisterId,
        expected_ty: TypeId,
    ) -> RegisterId {
        let input_node = builder.ir.node(input_register_id.into());
        let input_register = builder.ir.register(input_register_id);

        if let (&Type::Object(expected_class_id), &Type::Object(got_class_id))
        | (&Type::Class(expected_class_id), &Type::Class(got_class_id)) = (
            self.env.get_type(expected_ty),
            self.env.get_type(input_register.ty),
        ) {
            if self.is_subclass(expected_class_id, got_class_id) {
                // Subclass relationships do not need any implicit conversion logic.
                return input_register_id;
            } else {
                let inheritance_chain = self.note_inheritance_chain(got_class_id);
                let diagnostic = self
                    .type_mismatch(
                        input_node.span,
                        expected_ty,
                        input_register.ty,
                    )
                    .with_note(formatdoc!{"
                        note: `{got}` is not a subclass of `{expected}`. if you look at `{got}`'s inheritance chain...{chain}
                        note how it does not inherit from `{expected}` anywhere in the chain
                        therefore we cannot substitute `{expected}` with `{got}`, because it may be missing important items (functions, variables, etc.)",
                        got = self.env.class_name(got_class_id),
                        expected = self.env.class_name(expected_class_id),
                        chain = inheritance_chain,
                    });
                self.env.emit(diagnostic);
                return input_register_id;
            }
        }

        if !matches!(input_register.value, Value::Void)
            && expected_ty != TypeId::ERROR
            && input_register.ty != expected_ty
        {
            // Produce a generic type mismatch in any other case.
            let diagnostic = self.type_mismatch(input_node.span, expected_ty, input_register.ty);
            self.env.emit(diagnostic)
        }

        input_register_id
    }

    pub(super) fn expr_cast(
        &mut self,
        builder: &mut FunctionBuilder,
        outer: &cst::Expr,
        type_expr: &cst::Expr,
        type_id: TypeId,
        value_expr: &cst::Expr,
    ) -> RegisterId {
        match self.env.get_type(type_id) {
            // Error types should be skipped because we don't want weird ghost errors.
            Type::Error => {
                builder
                    .ir
                    .append_register(outer.span(), "error_type_cast", type_id, Value::Void)
            }

            // Casts to void are allowed to suppress unused local variable warnings.
            // If the user tries to use this expression anywhere in an expression, they'll get a
            // type mismatch error, so no need to do any extra checks here.
            Type::Void => {
                builder
                    .ir
                    .append_register(outer.span(), "void_cast", type_id, Value::Void)
            }

            Type::Primitive(primitive) => {
                self.expr_primitive_cast(builder, outer, type_expr, type_id, *primitive, value_expr)
            }
            Type::Object(_class_id) => {
                self.env.emit(
                    Diagnostic::error("object type casts are not yet supported")
                        .with_label(Label::primary(outer, "")),
                );
                builder.ir.append_register(
                    outer.span(),
                    "unsupported_object_cast",
                    type_id,
                    Value::Void,
                )
            }
            Type::Class(_class_id) => {
                self.env.emit(
                    Diagnostic::error("class type casts are not yet supported")
                        .with_label(Label::primary(outer, "")),
                );
                builder.ir.append_register(
                    outer.span(),
                    "unsupported_class_cast",
                    type_id,
                    Value::Void,
                )
            }
            Type::Struct { outer: _ } => {
                self.env.emit(
                    Diagnostic::error("struct type casts are not yet supported")
                        .with_label(Label::primary(outer, "")),
                );
                builder.ir.append_register(
                    outer.span(),
                    "unsupported_struct_cast",
                    type_id,
                    Value::Void,
                )
            }
            Type::Enum { outer: _ } => {
                self.env.emit(
                    Diagnostic::error("enum type casts are not yet supported")
                        .with_label(Label::primary(outer, "")),
                );
                builder.ir.append_register(
                    outer.span(),
                    "unsupported_enum_cast",
                    type_id,
                    Value::Void,
                )
            }

            Type::Array(_) => {
                self.env.emit(
                    Diagnostic::error(
                        "casting between dynamic array types is not supported",
                    )
                    .with_label(Label::primary(outer, ""))
                    .with_note("note: casting between array types like `Array<Int>` and `Array<Float>` is not supported by the VM and would be a very expensive operation"),
                );
                builder.ir.append_register(
                    outer.span(),
                    "unsupported_array_cast",
                    type_id,
                    Value::Void,
                )
            }
        }
    }

    fn expr_primitive_cast(
        &mut self,
        builder: &mut FunctionBuilder,
        outer: &cst::Expr,
        type_expr: &cst::Expr,
        to_type: TypeId,
        to_primitive: Primitive,
        value_expr: &cst::Expr,
    ) -> RegisterId {
        let value_register = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Matching(to_type),
            },
            value_expr,
        );
        let from_type = builder.ir.register(value_register).ty;

        if let &Type::Primitive(from_primitive) = self.env.get_type(from_type) {
            if let Some(cast) = Self::primitive_to_primitive_cast(from_primitive, to_primitive) {
                return builder.ir.append_register(
                    outer.span(),
                    "primitive_cast",
                    to_type,
                    Value::PrimitiveCast {
                        kind: cast,
                        value: value_register,
                    },
                );
            }
        }

        self.env.emit(
            Diagnostic::error("invalid cast")
                .with_label(Label::primary(
                    value_expr,
                    format!("from type `{}`", self.env.type_name(from_type)),
                ))
                .with_label(Label::primary(
                    type_expr,
                    format!("to type `{}`", self.env.type_name(to_type)),
                )),
        );
        builder.ir.append_register(
            outer.span(),
            "unsupported_primitive_cast",
            to_type,
            Value::Void,
        )
    }

    fn primitive_to_primitive_cast(
        from_primitive: Primitive,
        to_primitive: Primitive,
    ) -> Option<PrimitiveCast> {
        use Primitive::*;
        use PrimitiveCast::*;
        match (from_primitive, to_primitive) {
            (Bool, Byte) => Some(BoolToByte),
            (Bool, Float) => Some(BoolToFloat),
            (Bool, Int) => Some(BoolToInt),
            (Bool, String) => Some(BoolToString),
            (Byte, Bool) => Some(ByteToBool),
            (Byte, Float) => Some(ByteToFloat),
            (Byte, Int) => Some(ByteToInt),
            (Byte, String) => Some(ByteToString),
            (Float, Bool) => Some(FloatToBool),
            (Float, Byte) => Some(FloatToByte),
            (Float, Int) => Some(FloatToInt),
            (Float, String) => Some(FloatToString),
            (Int, Bool) => Some(IntToBool),
            (Int, Byte) => Some(IntToByte),
            (Int, Float) => Some(IntToFloat),
            (Int, String) => Some(IntToString),
            (Name, Bool) => Some(NameToBool),
            (Name, String) => Some(NameToString),
            (String, Bool) => Some(StringToBool),
            (String, Byte) => Some(StringToByte),
            (String, Float) => Some(StringToFloat),
            (String, Int) => Some(StringToInt),
            (String, Name) => Some(StringToName),
            _ => None,
        }
    }

    fn type_mismatch(
        &self,
        span: TokenSpan,
        expected_ty: TypeId,
        got_ty: TypeId,
    ) -> Diagnostic<Token> {
        Diagnostic::error("type mismatch")
            .with_label(Label::primary(&span, ""))
            .with_note(formatdoc! {"
                    expected `{}`
                         got `{}`
                ",
                self.env.type_name(expected_ty),
                self.env.type_name(got_ty)
            })
    }

    fn note_inheritance_chain(&mut self, class_id: ClassId) -> String {
        let mut inheritance_chain = String::new();
        let mut current_class_id = class_id;
        let mut is_first = true;
        while let Some(base) = self.super_class_id(current_class_id) {
            current_class_id = base;
            if is_first {
                inheritance_chain.push_str("\n  - it inherits from `");
            } else {
                inheritance_chain.push_str("\n  - which inherits from `")
            }
            inheritance_chain.push_str(self.env.class_name(current_class_id));
            inheritance_chain.push('`');
            is_first = false;
        }
        inheritance_chain
    }
}
