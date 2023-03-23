use std::collections::HashMap;

use muscript_foundation::ident::CaseInsensitive;
use muscript_syntax::cst::NamedItem;

use crate::{ClassId, Compiler, VarId};

use super::{UntypedClassPartitionsExt, Var, VarCst, VarFlags, VarKind};

#[derive(Debug, Default)]
pub struct ClassNamespace {
    pub all_var_names: Option<Vec<String>>,
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
                    let var = Var {
                        source_file_id,
                        name: cst.name(),
                        kind: match cst {
                            VarCst::Const(item_const) => VarKind::Const(item_const.value),
                            VarCst::Var(item_var) => VarKind::Var {
                                // NOTE: Process flags first, so that diagnostics are emitted
                                // from left to right.
                                flags: VarFlags::from_cst(
                                    self.env,
                                    self.sources,
                                    source_file_id,
                                    &item_var.specifiers,
                                ),
                                ty: self.type_id(source_file_id, class_id, &item_var.ty),
                            },
                        },
                    };
                    let var_id = self.env.register_var(var);
                    let namespace = self.env.class_namespace_mut(class_id);
                    namespace
                        .vars
                        .insert(CaseInsensitive::new(name.to_owned()), Some(var_id));
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

    pub fn all_var_names(&mut self, class_id: ClassId) -> &[String] {
        if self.env.class_namespace(class_id).all_var_names.is_none() {
            let all_var_names = if let Some(partitions) = self.untyped_class_partitions(class_id) {
                partitions
                    .iter()
                    .flat_map(|partition| partition.vars.keys().map(|ci| (**ci).clone()))
                    .collect()
            } else {
                vec![]
            };
            let namespace = self.env.class_namespace_mut(class_id);
            namespace.all_var_names = Some(all_var_names);
        }
        self.env
            .class_namespace(class_id)
            .all_var_names
            .as_ref()
            .unwrap()
    }

    pub fn class_vars(&mut self, class_id: ClassId) -> Vec<VarId> {
        // This clone is less than optimal, but in theory this function should only ever be called
        // once per class (ie. whenever the class is to be emitted,) so not much slowness should
        // happen. *In theory.*
        let all_var_names = self.all_var_names(class_id).to_owned();
        all_var_names
            .iter()
            .filter_map(|name| self.class_var(class_id, name))
            .collect()
    }
}
