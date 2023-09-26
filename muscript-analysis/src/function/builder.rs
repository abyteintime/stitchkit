use std::{borrow::Cow, collections::HashMap, ops::Deref};

use muscript_foundation::{ident::CaseInsensitive, source::SourceFileId};
use muscript_syntax::lexis::token::TokenSpan;

use crate::{
    ir::{BasicBlock, BasicBlockId, Ir, NodeId, RegisterId, Sink, Terminator, Value},
    ClassId, Environment, FunctionId, TypeId, VarId,
};

use super::Function;

pub struct FunctionBuilder {
    pub source_file_id: SourceFileId,
    pub class_id: ClassId,
    pub function_id: FunctionId,

    pub return_ty: TypeId,
    local_scopes: Vec<LocalScope>,

    pub ir: IrBuilder,
}

#[derive(Default)]
pub struct LocalScope {
    locals: HashMap<CaseInsensitive<String>, VarId>,
}

pub struct IrBuilder {
    ir: Ir,
    cursor: BasicBlockId,
}

/// # Lifecycle
impl FunctionBuilder {
    pub fn new(function_id: FunctionId, function: &Function, body_span: TokenSpan) -> Self {
        Self {
            source_file_id: function.source_file_id,
            class_id: function.class_id,
            function_id,
            return_ty: function.return_ty,
            local_scopes: vec![LocalScope::default()],
            ir: Ir::builder(body_span),
        }
    }

    pub fn into_ir(self) -> Ir {
        self.ir.into_ir()
    }
}

/// # Getters
impl FunctionBuilder {
    pub fn function<'a>(&self, env: &'a Environment) -> &'a Function {
        env.get_function(self.function_id)
    }
}

/// # Local stack
impl FunctionBuilder {
    pub fn push_local_scope(&mut self) {
        self.local_scopes.push(LocalScope::default());
    }

    pub fn pop_local_scope(&mut self) {
        let scope = self.local_scopes.pop();
        assert!(
            scope.is_some(),
            "unbalanced push_local_scope/pop_local_scope calls"
        );
    }

    /// If there was a variable declared in the same scope with the same name, returns it.
    pub fn add_local_to_scope(&mut self, name: &str, var_id: VarId) -> Option<VarId> {
        self.local_scopes
            .last_mut()
            .expect("there must be at least one scope to add variables into")
            .locals
            .insert(CaseInsensitive::new(name.to_owned()), var_id)
    }

    pub fn lookup_local(&self, name: &str) -> Option<VarId> {
        self.local_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.locals.get(CaseInsensitive::new_ref(name)))
            .copied()
    }
}

impl Ir {
    pub fn builder(begin_span: TokenSpan) -> IrBuilder {
        let mut ir = Ir::new();
        let begin = ir.create_basic_block(BasicBlock::new("begin", begin_span));
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
    pub fn append_basic_block(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        span: TokenSpan,
    ) -> BasicBlockId {
        let basic_block_id = self
            .ir
            .create_basic_block(BasicBlock::new(name.into(), span));
        self.set_cursor(basic_block_id);
        basic_block_id
    }

    #[must_use = "registers must be referenced by sinks to be evaluated"]
    pub fn append_register(
        &mut self,
        span: TokenSpan,
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

    pub fn append_sink(&mut self, span: TokenSpan, sink: Sink) -> NodeId {
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
