use std::{borrow::Cow, ops::Deref};

use muscript_foundation::source::{SourceFileId, Span};

use crate::{
    ir::{BasicBlock, BasicBlockId, Ir, NodeId, RegisterId, Sink, Terminator, Value},
    ClassId, Environment, FunctionId, TypeId, VarId,
};

use super::Function;

pub struct FunctionBuilder {
    pub(super) source_file_id: SourceFileId,
    pub(super) class_id: ClassId,
    pub(super) function_id: FunctionId,

    pub(super) return_ty: TypeId,

    pub(super) ir: IrBuilder,
}

pub struct IrBuilder {
    ir: Ir,
    cursor: BasicBlockId,
}

impl FunctionBuilder {
    pub fn function<'a>(&self, env: &'a Environment) -> &'a Function {
        env.get_function(self.function_id)
    }

    pub fn source_file_id(&self) -> SourceFileId {
        self.source_file_id
    }

    pub fn into_ir(self) -> Ir {
        self.ir.into_ir()
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

    #[must_use = "basic blocks must be linked to other basic blocks to be reachable"]
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
        ty: TypeId,
        value: Value,
    ) -> RegisterId {
        let register_id = self.ir.create_register(span, name.into(), ty, value);
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

impl Deref for IrBuilder {
    type Target = Ir;

    fn deref(&self) -> &Self::Target {
        &self.ir
    }
}
