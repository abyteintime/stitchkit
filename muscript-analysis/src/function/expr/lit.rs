use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::{
    cst,
    lexis::token::{FloatLit, IntLit, NameLit, StringLit},
};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    type_system::Type,
    Compiler, TypeId,
};

use super::{ExpectedType, ExprContext};

impl<'a> Compiler<'a> {
    pub(super) fn expr_lit(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        lit: &cst::Lit,
    ) -> RegisterId {
        match lit {
            cst::Lit::Bool(lit) => self.expr_lit_bool(builder, lit),
            cst::Lit::Int(lit) => self.expr_lit_int(builder, context, lit),
            cst::Lit::Float(lit) => self.expr_lit_float(builder, lit),
            cst::Lit::String(lit) => self.expr_lit_string(builder, lit),
            cst::Lit::Name(lit) => self.expr_lit_name(builder, lit),
            cst::Lit::None(lit) => self.expr_lit_none(builder, context, lit),
        }
    }

    fn expr_lit_bool(&mut self, builder: &mut FunctionBuilder, lit: &cst::BoolLit) -> RegisterId {
        builder.ir.append_register(
            lit.span(),
            "lit_bool",
            TypeId::BOOL,
            match lit {
                cst::BoolLit::True(_) => Value::Bool(true),
                cst::BoolLit::False(_) => Value::Bool(false),
            },
        )
    }

    fn expr_lit_int(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        lit: &IntLit,
    ) -> RegisterId {
        // NOTE: Int literals coerce to floats automatically.
        let input = self.sources.source(builder.source_file_id);
        let type_id = if context.expected_type.is_exactly(TypeId::FLOAT) {
            TypeId::FLOAT
        } else if context.expected_type.is_exactly(TypeId::BYTE) {
            TypeId::BYTE
        } else {
            TypeId::INT
        };
        let i = lit.parse(input, self.env, builder.source_file_id);
        builder.ir.append_register(
            lit.span,
            "lit_int",
            type_id,
            if type_id == TypeId::FLOAT {
                Value::Float(i as f32)
            } else if type_id == TypeId::BYTE {
                let byte = u8::try_from(i);
                if byte.is_err() {
                    self.env.emit(
                        Diagnostic::error(builder.source_file_id, "byte value out of range")
                            .with_label(Label::primary(lit.span, ""))
                            .with_note("note: byte literals must fit in the range [0, 255]"),
                    )
                }
                Value::Byte(byte.unwrap_or(0))
            } else {
                Value::Int(i)
            },
        )
    }

    fn expr_lit_float(&mut self, builder: &mut FunctionBuilder, lit: &FloatLit) -> RegisterId {
        let input = self.sources.source(builder.source_file_id);
        let f = lit.parse(input, self.env, builder.source_file_id);
        builder
            .ir
            .append_register(lit.span, "lit_float", TypeId::FLOAT, Value::Float(f))
    }

    fn expr_lit_string(&mut self, builder: &mut FunctionBuilder, lit: &StringLit) -> RegisterId {
        let input = self.sources.source(builder.source_file_id);
        let s = lit.parse(input, self.env, builder.source_file_id);
        builder
            .ir
            .append_register(lit.span, "lit_string", TypeId::STRING, Value::String(s))
    }

    fn expr_lit_name(&mut self, builder: &mut FunctionBuilder, lit: &NameLit) -> RegisterId {
        let input = self.sources.source(builder.source_file_id);
        let n = lit.parse(input).to_string();
        builder
            .ir
            .append_register(lit.span, "lit_name", TypeId::NAME, Value::Name(n))
    }

    fn expr_lit_none(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        lit: &cst::KNone,
    ) -> RegisterId {
        let ty = match context.expected_type {
            // NOTE: `none` is always an object literal. Therefore we need to ensure the returned
            // type is either an `Object` subclass, or `Object` itself.
            ExpectedType::Matching(type_id) => {
                let ty = self.env.get_type(type_id);
                match ty {
                    Type::Object(_) => type_id,
                    _ => TypeId::OBJECT,
                }
            }
            ExpectedType::Any => TypeId::OBJECT,
        };
        builder
            .ir
            .append_register(lit.span, "lit_none", ty, Value::None)
    }
}
