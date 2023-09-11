//! Constant evaluation engine (IR interpreter.)

use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::{SourceFileId, Span},
};

use crate::{function::builder::IrBuilder, TypeId};

use super::{Ir, NodeKind, RegisterId, Terminator, Value};

/// Constant expression value. Corresponds directly to a subset of [`Value`] variants.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// Produced when there is an evaluation error.
    Void,
    Bool(bool),
    Byte(u8),
    Int(i32),
    Float(f32),
    String(String),
    Name(String),
}

impl Constant {
    pub fn type_id(&self) -> TypeId {
        match self {
            Constant::Void => TypeId::VOID,
            Constant::Bool(_) => TypeId::BOOL,
            Constant::Byte(_) => TypeId::BYTE,
            Constant::Int(_) => TypeId::INT,
            Constant::Float(_) => TypeId::FLOAT,
            Constant::String(_) => TypeId::STRING,
            Constant::Name(_) => TypeId::NAME,
        }
    }

    pub fn append_to(&self, ir: &mut IrBuilder, span: Span, name: &str) -> RegisterId {
        ir.append_register(
            span,
            name.to_owned(),
            self.type_id(),
            match self {
                Constant::Void => Value::Void,
                &Constant::Bool(x) => Value::Bool(x),
                &Constant::Byte(x) => Value::Byte(x),
                &Constant::Int(x) => Value::Int(x),
                &Constant::Float(x) => Value::Float(x),
                Constant::String(x) => Value::String(x.clone()),
                Constant::Name(x) => Value::Name(x.clone()),
            },
        )
    }
}

pub fn interpret(
    diagnostics: &mut dyn DiagnosticSink,
    source_file_id: SourceFileId,
    ir: &Ir,
) -> Constant {
    let block = &ir.basic_blocks[0];

    // NOTE: make this a loop whenever you add support for branching.
    // Not a loop right now because that would be misleading (and clippy doesn't like it.)
    // loop {

    // Evaluate side effects
    for &node_id in &block.flow {
        let node = ir.node(node_id);
        if let NodeKind::Sink(_) = node.kind {
            diagnostics.emit(cannot_evaluate_at_compile_time(
                source_file_id,
                CannotEvaluateAtCompileTime::Statement,
                node.span,
            ));
            return Constant::Void;
        }
    }

    // Evaluate the terminator
    match &block.terminator {
        &Terminator::Return(register_id) => {
            eval_register(diagnostics, source_file_id, ir, register_id)
        }

        // For now we disallow any sort of branching.
        Terminator::Goto(_) => {
            // When implementing this in the future, remember to set up some mechanism of
            // "branching fuel" - a limit to how many backward branches can be taken, so as to
            // avoid compiling indefinitely.
            diagnostics.emit(
                Diagnostic::error(source_file_id, "loops cannot be evaluated at compile time")
                    .with_label(Label::primary(block.span, "")),
            );
            Constant::Void
        }
        Terminator::GotoIf { .. } => {
            diagnostics.emit(
                Diagnostic::error(
                    source_file_id,
                    "conditional branches (`if`s and `?:`) cannot be evaluated at compile time",
                )
                .with_label(Label::primary(block.span, "")),
            );
            Constant::Void
        }

        Terminator::Unreachable => {
            diagnostics.emit(
                Diagnostic::bug(source_file_id, "unreachable IR reached")
                    .with_label(Label::primary(block.span, ""))
                    .with_note("note: this is a bug, please report it at <https://github.com/abyteintime/stitchkit>"),
            );
            Constant::Void
        }
    }

    // }
}

fn eval_register(
    diagnostics: &mut dyn DiagnosticSink,
    source_file_id: SourceFileId,
    ir: &Ir,
    register_id: RegisterId,
) -> Constant {
    let register = ir.register(register_id);
    match &register.value {
        Value::Void => Constant::Void,
        &Value::Bool(x) => Constant::Bool(x),
        &Value::Byte(x) => Constant::Byte(x),
        &Value::Int(x) => Constant::Int(x),
        &Value::Float(x) => Constant::Float(x),
        Value::String(x) => Constant::String(x.clone()),
        _ => {
            diagnostics.emit(cannot_evaluate_at_compile_time(
                source_file_id,
                CannotEvaluateAtCompileTime::Expression,
                ir.node(register_id.into()).span,
            ));
            Constant::Void
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CannotEvaluateAtCompileTime {
    Expression,
    Statement,
}

fn cannot_evaluate_at_compile_time(
    source_file_id: SourceFileId,
    kind: CannotEvaluateAtCompileTime,
    span: Span,
) -> Diagnostic {
    Diagnostic::error(
        source_file_id,
        match kind {
            CannotEvaluateAtCompileTime::Expression => {
                "expression cannot be evaluated at compile time"
            }
            CannotEvaluateAtCompileTime::Statement => {
                "statement cannot be evaluated at compile time"
            }
        },
    )
    .with_label(Label::primary(span, ""))
    .with_note("note: compile-time evaluation only supports constants right now")
}
