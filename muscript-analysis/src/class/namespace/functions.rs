use muscript_foundation::ident::CaseInsensitive;

use crate::{partition::UntypedClassPartitionsExt, ClassId, Compiler, FunctionId};

/// # Functions
impl<'a> Compiler<'a> {
    pub fn function_in_class(&mut self, class_id: ClassId, name: &str) -> Option<FunctionId> {
        let namespace = self.env.class_namespace(class_id);
        if !namespace
            .functions
            .contains_key(CaseInsensitive::new_ref(name))
        {
            if let Some(partitions) = self.untyped_class_partitions(class_id) {
                if let Some((source_file_id, cst)) = partitions.find_function(name) {
                    let function_id = self.analyze_function(source_file_id, class_id, name, cst);
                    let namespace = self.env.class_namespace_mut(class_id);
                }
            }
        }
        let namespace = self.env.class_namespace_mut(class_id);
        namespace
            .functions
            .get(CaseInsensitive::new_ref(name))
            .and_then(|x| x.as_ref())
            .copied()
    }
}
