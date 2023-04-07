use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};
use muscript_syntax::cst;

use crate::{
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

use super::builder::FunctionBuilder;

#[derive(Debug, Clone)]
pub struct ExprContext {
    pub expected_type: ExpectedType,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedType {
    Matching(TypeId),
    Any,
}

impl ExpectedType {
    fn to_type_id(self) -> TypeId {
        match self {
            ExpectedType::Matching(type_id) => type_id,
            ExpectedType::Any => TypeId::VOID,
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn expr(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        expr: &cst::Expr,
    ) -> (TypeId, RegisterId) {
        match expr {
            _ => {
                self.env.emit(
                    Diagnostic::error(builder.source_file_id, "unsupported expression")
                        .with_label(Label::primary(expr.span(), ""))
                        .with_note("note: MuScript is still unfinished; you can help contribute at <https://github.com/abyteintime/stitchkit>")
                );
                let void = builder
                    .ir
                    .append_register(expr.span(), "unsupported", Value::Void);
                (context.expected_type.to_type_id(), void)
            }
        }
    }
}
