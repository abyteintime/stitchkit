use std::collections::HashMap;

use crate::{environment::ClassId, CompileError, Compiler, VarId};

#[derive(Debug, Clone)]
pub struct Package {
    pub classes: HashMap<ClassId, PackagedClass>,
}

#[derive(Debug, Clone)]
pub struct PackagedClass {
    pub vars: Vec<VarId>,
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
                },
            );
        }

        Ok(Self { classes })
    }
}
