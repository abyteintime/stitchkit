use muscript_foundation::ident::CaseInsensitive;
use tracing::{info_span, trace};

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
                if let Some(partition_index) = partitions.index_of_partition_with_function(name) {
                    let partitions = self.untyped_class_partitions_for_theft(class_id).unwrap();
                    let partition = &mut partitions[partition_index];
                    let (function_name, stolen_cst) = partition
                        .functions
                        .remove_entry(CaseInsensitive::new_ref(name))
                        .expect("index_of_partition_with_function returned Some for a reason");

                    let function_id = self.analyze_function_signature(class_id, name, &stolen_cst);

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
        let _span = info_span!(
            "class_functions",
            ?class_id,
            class_name = self.env.class_name(class_id)
        )
        .entered();

        let all_function_names = self.all_function_names(class_id).to_owned();
        all_function_names
            .iter()
            .filter_map(|name| self.function_in_class(class_id, name))
            .collect()
    }

    pub fn lookup_function(&mut self, class_id: ClassId, name: &str) -> Option<FunctionId> {
        let _span = info_span!("lookup_function", ?class_id, name).entered();

        // TODO: Speed this up via memoization? Walking the inheritance hierarchy could be
        // a bit slow.
        if let Some(function_id) = self.function_in_class(class_id, name) {
            trace!("found function");
            Some(function_id)
        } else if let Some(parent_class) = self.super_class_id(class_id) {
            trace!(?parent_class, "walking up to parent class");
            self.lookup_function(parent_class, name)
        } else {
            trace!("did not find anything");
            None
        }
    }
}
