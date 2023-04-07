use std::borrow::Cow;

use muscript_foundation::source::{SourceFileId, Span};

use crate::{
    ir::{BasicBlock, BasicBlockId, Ir, NodeId, RegisterId, Sink, Terminator, Value},
    ClassId, TypeId, VarId,
};

use super::{Function, FunctionFlags, FunctionImplementation, Param};

pub struct FunctionBuilder {
    pub(super) source_file_id: SourceFileId,
    pub(super) class_id: ClassId,
    pub(super) flags: FunctionFlags,
    pub(super) return_ty: TypeId,
    pub(super) params: Vec<Param>,
    pub(super) ir: IrBuilder,
}

pub struct IrBuilder {
    ir: Ir,
    cursor: BasicBlockId,
}

impl FunctionBuilder {
    pub fn into_function(
        self,
        mangled_name: String,
        implementation: FunctionImplementation,
    ) -> Function {
        Function {
            source_file_id: self.source_file_id,
            mangled_name,
            ir: self.ir.into_ir(),
            return_ty: self.return_ty,
            params: self.params,
            flags: self.flags,
            implementation,
        }
    }
}

impl Ir {
    pub fn builder() -> IrBuilder {
        let mut ir = Ir::new();
        let begin = ir.create_basic_block(BasicBlock::new("begin"));
        IrBuilder { ir, cursor: begin }
    }
}

impl IrBuilder {
    #[must_use]
    pub fn cursor(&self) -> BasicBlockId {
        self.cursor
    }

    pub fn set_cursor(&mut self, basic_block_id: BasicBlockId) {
        self.cursor = basic_block_id;
    }

    pub fn add_local(&mut self, var_id: VarId) {
        self.ir.add_local(var_id);
    }

    #[must_use = "basic blocks must be linked to other basic blocks to belong to be reachable"]
    pub fn append_basic_block(&mut self, name: impl Into<Cow<'static, str>>) -> BasicBlockId {
        let basic_block_id = self.ir.create_basic_block(BasicBlock::new(name.into()));
        self.set_cursor(basic_block_id);
        basic_block_id
    }

    #[must_use = "registers must be referenced by sinks to be evaluated"]
    pub fn append_register(
        &mut self,
        span: Span,
        name: impl Into<Cow<'static, str>>,
        value: Value,
    ) -> RegisterId {
        let register_id = self.ir.create_register(span, name.into(), value);
        self.ir
            .basic_block_mut(self.cursor())
            .flow
            .push(register_id.into());
        register_id
    }

    pub fn append_sink(&mut self, span: Span, sink: Sink) -> NodeId {
        let node_id = self.ir.create_sink(span, sink);
        self.ir.basic_block_mut(self.cursor()).flow.push(node_id);
        node_id
    }

    pub fn set_terminator(&mut self, terminator: Terminator) {
        self.ir.basic_block_mut(self.cursor()).terminator = terminator;
    }

    pub fn into_ir(self) -> Ir {
        self.ir
    }
}
