use std::collections::HashMap;

use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, Diagnostic, DiagnosticSink},
    ident::CaseInsensitive,
};

use crate::{
    class::{ClassNamespace, UntypedClassPartition, Var},
    type_system::{Primitive, Type},
    Compiler,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(u32);

#[derive(Debug, Default)]
pub struct Environment {
    pub diagnostics: Vec<Diagnostic>,

    class_ids_by_name: HashMap<CaseInsensitive<String>, ClassId>,
    class_names_by_id: Vec<CaseInsensitive<String>>,
    class_namespaces_by_id: Vec<ClassNamespace>,

    untyped_class_partitions: HashMap<ClassId, Option<Vec<UntypedClassPartition>>>,

    types: Vec<Type>,
    vars: Vec<Var>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            diagnostics: vec![],
            class_ids_by_name: HashMap::new(),
            class_names_by_id: vec![],
            class_namespaces_by_id: vec![],
            untyped_class_partitions: HashMap::new(),
            types: vec![],
            vars: vec![],
        };

        env.register_type(Type::Error);
        // NOTE: Order matters here! The TypeIds must match exactly those returned by Primitive::id.
        env.register_type(Type::Primitive(Primitive::Bool));
        env.register_type(Type::Primitive(Primitive::Byte));
        env.register_type(Type::Primitive(Primitive::Int));
        env.register_type(Type::Primitive(Primitive::Float));
        env.register_type(Type::Primitive(Primitive::String));
        env.register_type(Type::Primitive(Primitive::Name));
        env
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// # Class registry
impl Environment {
    pub fn get_or_create_class(&mut self, class_name: &str) -> ClassId {
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
            self.class_namespaces_by_id.push(Default::default());
            id
        }
    }

    pub fn class_name(&self, class_id: ClassId) -> &str {
        self.class_names_by_id
            .get(class_id.0 as usize)
            .map(|x| x.as_ref())
            .expect("invalid class ID passed to class_name")
    }

    pub fn class_namespace(&self, class_id: ClassId) -> &ClassNamespace {
        self.class_namespaces_by_id
            .get(class_id.0 as usize)
            .expect("invalid class ID passed to class_namespace")
    }

    pub fn class_namespace_mut(&mut self, class_id: ClassId) -> &mut ClassNamespace {
        self.class_namespaces_by_id
            .get_mut(class_id.0 as usize)
            .expect("invalid class ID passed to class_namespace_mut")
    }
}

/// # Miscellaneous registries
///
/// These are lumped into one category because they don't provide any extra functionality beyond
/// registering and obtaining their elements.
impl Environment {
    pub fn register_type(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    pub fn get_type(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }

    pub fn register_var(&mut self, var: Var) -> VarId {
        let id = VarId(self.vars.len() as u32);
        self.vars.push(var);
        id
    }

    pub fn get_var(&self, id: VarId) -> &Var {
        &self.vars[id.0 as usize]
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
                let partitions: Vec<_> = class_csts
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
                UntypedClassPartition::check_namespace_coherence(
                    &mut diagnostics,
                    self.sources,
                    &partitions,
                );
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

impl TypeId {
    pub const ERROR: Self = Self(0);
}

impl Primitive {
    pub fn id(&self) -> TypeId {
        match self {
            Primitive::Bool => TypeId(1),
            Primitive::Byte => TypeId(2),
            Primitive::Int => TypeId(3),
            Primitive::Float => TypeId(4),
            Primitive::String => TypeId(5),
            Primitive::Name => TypeId(6),
        }
    }
}
