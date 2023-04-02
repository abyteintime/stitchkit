use muscript_foundation::source::SourceFileId;
use muscript_syntax::cst;

use crate::{ir::Ir, ClassId, Compiler, FunctionId, TypeId};

pub mod mangling;

#[derive(Clone)]
pub struct Function {
    pub source_file_id: SourceFileId,
    pub mangled_name: String,
    pub ir: Ir,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").finish_non_exhaustive()
    }
}

impl<'a> Compiler<'a> {
    /// Analyzes a function from CST.
    pub(crate) fn analyze_function(
        &mut self,
        source_file_id: SourceFileId,
        class_id: ClassId,
        name: &str,
        cst: &cst::ItemFunction,
    ) -> FunctionId {
        let return_ty = cst
            .return_ty
            .as_ref()
            .map(|ty| self.type_id(source_file_id, class_id, ty))
            .unwrap_or(TypeId::VOID);
        self.env.register_function(Function {
            source_file_id,
            mangled_name: name.to_owned(),
            ir: Ir {
                return_ty,
                locals: vec![],
                param_count: 0,
                nodes: vec![],
                basic_blocks: vec![],
            },
        })
    }
}
