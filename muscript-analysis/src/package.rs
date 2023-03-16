use std::collections::HashMap;

use crate::{
    class::Class,
    environment::{ClassId, Environment},
    source::CompilerInput,
    CompileError,
};

pub struct Package {
    pub classes: HashMap<ClassId, Class>,
}

impl Package {
    /// Compiles a package from the given set of classes.
    pub fn compile(
        env: &mut Environment,
        sources: &dyn CompilerInput,
        classes: &[ClassId],
    ) -> Result<Self, CompileError> {
        for &class in classes {
            // Need to convert to an owned string here, because otherwise we get a mutable and
            // immutable borrow at the same time. This shouldn't be that terrible for performance
            // though, since compared to the amount of computation we do later it's a small thing.
            let class_name = env.class_name(class).to_owned();
            let class_csts = sources.class_sources(&class_name, env);
            dbg!(class_csts);
        }

        Err(CompileError) // TODO
    }
}
