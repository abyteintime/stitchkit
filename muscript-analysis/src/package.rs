use std::collections::HashMap;

use crate::{class::UntypedClassPartition, environment::ClassId, CompileError, Compiler};

#[derive(Debug, Clone)]
pub struct Package {
    pub classes: HashMap<ClassId, Vec<UntypedClassPartition>>,
}

impl Package {
    /// Compiles a package from the given set of classes.
    pub fn compile(
        compiler: &mut Compiler<'_>,
        class_ids: &[ClassId],
    ) -> Result<Self, CompileError> {
        let mut classes = HashMap::new();
        let mut error = false;
        for &class_id in class_ids {
            if let Some(untyped_class_partitions) = compiler.untyped_class_partitions(class_id) {
                classes.insert(class_id, untyped_class_partitions.to_owned());
            } else {
                error = true;
            }
        }

        if error {
            Err(CompileError)
        } else {
            Ok(Self { classes })
        }
    }
}
