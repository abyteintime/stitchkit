use muscript_foundation::source::Spanned;
use muscript_syntax::{
    cst,
    lexis::token::{FloatLit, IntLit, StringLit},
};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

use super::ExprContext;

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
            cst::Lit::Name(_) => todo!(),
            cst::Lit::None(_) => todo!(),
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
        } else {
            TypeId::INT
        };
        let i = lit.parse(input, self.env, builder.source_file_id);
        builder.ir.append_register(
            lit.span,
            "lit_int",
            type_id,
            if type_id == TypeId::FLOAT {
                Value::Int(i)
            } else {
                Value::Float(i as f32)
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
}
