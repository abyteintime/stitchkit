use std::collections::HashMap;

use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink},
    ident::CaseInsensitive,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(u32);

#[derive(Debug, Default)]
pub struct Environment {
    pub diagnostics: Vec<Diagnostic>,

    class_ids_by_name: HashMap<CaseInsensitive<String>, ClassId>,
    class_names_by_id: Vec<CaseInsensitive<String>>,
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
