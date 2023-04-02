use std::collections::HashMap;

use muscript_foundation::errors::pipe_all_diagnostics_into;

use crate::{environment::ClassId, CompileError, Compiler, FunctionId, VarId};

#[derive(Debug, Clone)]
pub struct Package {
    pub classes: HashMap<ClassId, PackagedClass>,
}

#[derive(Debug, Clone)]
pub struct PackagedClass {
    pub vars: Vec<VarId>,
    pub functions: Vec<FunctionId>,
}

impl Package {
    /// Compiles a package from the given set of classes.
    pub fn compile(
        compiler: &mut Compiler<'_>,
        class_ids: &[ClassId],
    ) -> Result<Self, CompileError> {
        let mut classes = HashMap::new();
        for &class_id in class_ids {
            classes.insert(
                class_id,
                PackagedClass {
                    vars: compiler.class_vars(class_id),
                    functions: compiler.class_functions(class_id),
                },
            );

            let mut support_diagnostics = vec![];
            for partition in compiler
                .untyped_class_partitions(class_id)
                .into_iter()
                .flatten()
            {
                partition.check_item_support(&mut support_diagnostics);
            }
            pipe_all_diagnostics_into(compiler.env, support_diagnostics);
        }

        Ok(Self { classes })
    }
}
