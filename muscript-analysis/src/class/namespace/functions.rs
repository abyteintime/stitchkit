use muscript_foundation::ident::CaseInsensitive;

use crate::{
    partition::{UntypedClassPartition, UntypedClassPartitionsExt},
    ClassId, Compiler, FunctionId,
};

/// # Functions
impl<'a> Compiler<'a> {
    pub fn function_in_class(&mut self, class_id: ClassId, name: &str) -> Option<FunctionId> {
        let namespace = self.env.class_namespace(class_id);
        if !namespace
            .functions
            .contains_key(CaseInsensitive::new_ref(name))
        {
            if let Some(partitions) = self.untyped_class_partitions(class_id) {
                if let Some(partition_index) = partitions.index_of_partition_with_function(name) {
                    let partitions = self.untyped_class_partitions_for_theft(class_id).unwrap();
                    let partition = &mut partitions[partition_index];
                    let (function_name, stolen_cst) = partition
                        .functions
                        .remove_entry(CaseInsensitive::new_ref(name))
                        .expect("index_of_partition_with_function returned Some for a reason");
                    let &mut UntypedClassPartition { source_file_id, .. } = partition;

                    let function_id = self.analyze_function_signature(
                        source_file_id,
                        class_id,
                        name,
                        &stolen_cst,
                    );

                    // As per our "theft" contract (which does not really involve theft - this is a
                    // pacifist run) - give it back.
                    let partitions = self.untyped_class_partitions_for_theft(class_id).unwrap();
                    let partition = &mut partitions[partition_index];
                    partition.functions.insert(function_name, stolen_cst);

                    let namespace = self.env.class_namespace_mut(class_id);
                    namespace
                        .functions
                        .insert(CaseInsensitive::new(name.to_owned()), Some(function_id));
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

    pub fn all_function_names(&mut self, class_id: ClassId) -> &[String] {
        if self
            .env
            .class_namespace(class_id)
            .all_function_names
            .is_none()
        {
            let all_function_names =
                if let Some(partitions) = self.untyped_class_partitions(class_id) {
                    partitions
                        .iter()
                        .flat_map(|partition| partition.functions.keys().map(|ci| (**ci).clone()))
                        .collect()
                } else {
                    vec![]
                };
            let namespace = self.env.class_namespace_mut(class_id);
            namespace.all_function_names = Some(all_function_names);
        }
        self.env
            .class_namespace(class_id)
            .all_function_names
            .as_ref()
            .unwrap()
    }

    pub fn class_functions(&mut self, class_id: ClassId) -> Vec<FunctionId> {
        let all_function_names = self.all_function_names(class_id).to_owned();
        all_function_names
            .iter()
            .filter_map(|name| self.function_in_class(class_id, name))
            .collect()
    }

    pub fn lookup_function(&mut self, class_id: ClassId, name: &str) -> Option<FunctionId> {
        // TODO: Speed this up via memoization? Walking the inheritance hierarchy could be
        // a bit slow.
        if let Some(function_id) = self.function_in_class(class_id, name) {
            Some(function_id)
        } else if let Some(parent_class) = self.super_class_id(class_id) {
            self.function_in_class(parent_class, name)
        } else {
            None
        }
    }
}
