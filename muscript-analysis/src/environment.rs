use std::collections::HashMap;

use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, Diagnostic, DiagnosticSink},
    ident::CaseInsensitive,
};

use crate::{class::UntypedClassPartition, Compiler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(u32);

#[derive(Debug, Default)]
pub struct Environment {
    pub diagnostics: Vec<Diagnostic>,

    class_ids_by_name: HashMap<CaseInsensitive<String>, ClassId>,
    class_names_by_id: Vec<CaseInsensitive<String>>,

    untyped_class_partitions: HashMap<ClassId, Option<Vec<UntypedClassPartition>>>,
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn allocate_class_id(&mut self, class_name: &str) -> ClassId {
        let class_name = CaseInsensitive::new(class_name.to_owned());
        if let Some(id) = self.class_ids_by_name.get(&class_name) {
            *id
        } else {
            let id = ClassId(
                self.class_ids_by_name
                    .len()
                    .try_into()
                    .expect("too many classes declared"),
            );
            self.class_ids_by_name.insert(class_name.clone(), id);
            self.class_names_by_id.push(class_name);
            id
        }
    }

    pub fn class_name(&self, id: ClassId) -> &str {
        self.class_names_by_id
            .get(id.0 as usize)
            .map(|x| x.as_ref())
            .expect("invalid class ID passed to class_name")
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl DiagnosticSink for Environment {
    fn emit(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

impl<'a> Compiler<'a> {
    /// Returns the set of untyped partitions for the class with the given ID, or `None` if the
    /// class represented by the ID does not exist.
    pub fn untyped_class_partitions(
        &mut self,
        class_id: ClassId,
    ) -> Option<&[UntypedClassPartition]> {
        if self.env.untyped_class_partitions.get(&class_id).is_none() {
            let class_name = self.env.class_name(class_id).to_owned();
            if let Some(class_csts) = self.input.class_sources(&class_name, self.env) {
                let mut diagnostics = vec![];
                let partitions = class_csts
                    .into_iter()
                    .map(|(source_file_id, cst)| {
                        UntypedClassPartition::from_cst(
                            &mut diagnostics,
                            self.sources,
                            source_file_id,
                            cst,
                        )
                    })
                    .collect();
                pipe_all_diagnostics_into(self.env, diagnostics);
                self.env
                    .untyped_class_partitions
                    .insert(class_id, Some(partitions));
            }
        }
        self.env
            .untyped_class_partitions
            .get(&class_id)
            .and_then(|x| x.as_ref())
            .map(|x| x.as_slice())
    }
}
