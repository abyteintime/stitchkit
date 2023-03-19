use std::collections::HashMap;

use muscript_foundation::ident::CaseInsensitive;
use muscript_syntax::cst::NamedItem;

use crate::{ClassId, Compiler, VarId};

use super::{UntypedClassPartitionsExt, Var};

#[derive(Debug, Default)]
pub struct ClassNamespace {
    pub vars: HashMap<CaseInsensitive<String>, Option<VarId>>,
}

impl<'a> Compiler<'a> {
    pub fn class_var(&mut self, class_id: ClassId, name: &str) -> Option<VarId> {
        let namespace = self.env.class_namespace(class_id);
        if !namespace.vars.contains_key(CaseInsensitive::new_ref(name)) {
            if let Some(partitions) = self.untyped_class_partitions(class_id) {
                if let Some((source_file_id, cst)) = partitions.find_var(name) {
                    // Cloning here is kind of inefficient, but otherwise we hold a reference
                    // to the class partitions and thus we cannot register variables within the
                    // environment.
                    let cst = cst.clone();
                    let var = self.env.register_var(Var {
                        source_file_id,
                        name: cst.name(),
                    });
                    let namespace = self.env.class_namespace_mut(class_id);
                    namespace
                        .vars
                        .insert(CaseInsensitive::new(name.to_owned()), Some(var));
                }
            }
        }
        let namespace = self.env.class_namespace_mut(class_id);
        namespace
            .vars
            .get(CaseInsensitive::new_ref(name))
            .and_then(|x| x.as_ref())
            .copied()
    }
}
