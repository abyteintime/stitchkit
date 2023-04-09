use muscript_foundation::source::Spanned;
use muscript_syntax::cst;

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
        let input = self.sources.source(builder.source_file_id);
        match lit {
            cst::Lit::None(_) => todo!(),
            cst::Lit::Bool(b) => builder.ir.append_register(
                b.span(),
                "lit_bool",
                TypeId::BOOL,
                match b {
                    cst::BoolLit::True(_) => Value::Bool(true),
                    cst::BoolLit::False(_) => Value::Bool(false),
                },
            ),
            cst::Lit::Int(x) => {
                // NOTE: Int literals coerce to floats automatically.
                let type_id = if context.expected_type.is_exactly(TypeId::FLOAT) {
                    TypeId::FLOAT
                } else {
                    TypeId::INT
                };
                let i = x.parse(input, self.env, builder.source_file_id);
                builder.ir.append_register(
                    x.span,
                    "lit_int",
                    type_id,
                    if type_id == TypeId::FLOAT {
                        Value::Int(i)
                    } else {
                        Value::Float(i as f32)
                    },
                )
            }
            cst::Lit::Float(_) => todo!(),
            cst::Lit::String(_) => todo!(),
            cst::Lit::Name(_) => todo!(),
        }
    }
}
