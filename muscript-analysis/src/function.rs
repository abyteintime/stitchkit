use muscript_foundation::source::SourceFileId;
use muscript_syntax::cst;

use crate::{ir::Ir, ClassId, Compiler, FunctionId};

pub mod mangling;

#[derive(Clone)]
pub struct Function {
    pub ir: Ir,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").finish_non_exhaustive()
    }
}

impl<'a> Compiler<'a> {
    /// Analyzes a function from CST.
    pub fn analyze_function(
        &mut self,
        source_file_id: SourceFileId,
        class_id: ClassId,
        name: &str,
        cst: &cst::ItemFunction,
    ) -> FunctionId {
        todo!()
    }
}
