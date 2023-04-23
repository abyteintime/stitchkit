use std::collections::HashMap;

use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    source::SourceFileId,
};
use muscript_syntax::{cst, lexis::token::Ident};
use tracing::trace;

use crate::{
    class::{ClassNamespace, Var},
    function::Function,
    ir::Ir,
    partition::UntypedClassPartition,
    type_system::{lookup::TypeSource, Primitive, Type, TypeName},
    Compiler,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u32);

#[derive(Debug, Default)]
pub struct Environment {
    pub diagnostics: Vec<Diagnostic>,

    class_ids_by_name: HashMap<CaseInsensitive<String>, ClassId>,
    class_names_by_id: Vec<CaseInsensitive<String>>,
    class_namespaces_by_id: Vec<ClassNamespace>,

    untyped_class_partitions: HashMap<ClassId, Option<Vec<UntypedClassPartition>>>,

    types: Vec<Type>,
    vars: Vec<Var>,
    functions: Vec<Function>,

    global_type_ids_by_name: HashMap<TypeName, TypeId>,
    scoped_type_ids_by_name: HashMap<(ClassId, TypeName), TypeId>,
    type_names_by_id: Vec<TypeName>,

    irs_by_function_id: HashMap<FunctionId, Ir>,
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
            functions: vec![],
            global_type_ids_by_name: HashMap::new(),
            scoped_type_ids_by_name: HashMap::new(),
            type_names_by_id: vec![],
            irs_by_function_id: HashMap::new(),
        };
        env.register_fundamental_types();
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

    pub fn get_class(&self, class_name: &str) -> Option<ClassId> {
        self.class_ids_by_name
            .get(CaseInsensitive::new_ref(class_name))
            .copied()
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

    pub fn untyped_class_partitions(&self, class_id: ClassId) -> Option<&[UntypedClassPartition]> {
        self.untyped_class_partitions
            .get(&class_id)
            .and_then(|x| x.as_ref())
            .map(|x| x.as_slice())
    }
}

/// # Class registry
impl<'a> Compiler<'a> {
    /// Look up a class ID from an identifier.
    ///
    /// Returns `None` and emits a diagnostic if the class cannot be found.
    pub fn lookup_class(&mut self, source_file_id: SourceFileId, ident: Ident) -> Option<ClassId> {
        let name = self.sources.span(source_file_id, &ident);
        if self.input.class_exists(name) {
            Some(self.env.get_or_create_class(name))
        } else {
            self.env.emit(
                Diagnostic::error(source_file_id, format!("class `{name}` does not exist"))
                    .with_label(Label::primary(ident.span, "")),
            );
            None
        }
    }
}

/// # Type registry
impl Environment {
    pub fn register_type(&mut self, name: TypeName, ty: Type) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        trace!(%name, ?id, "registering type");
        self.types.push(ty);
        self.type_names_by_id.push(name);
        id
    }

    pub fn get_type(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }

    pub fn type_name(&self, id: TypeId) -> &TypeName {
        &self.type_names_by_id[id.0 as usize]
    }

    pub fn class_type(&mut self, class_id: ClassId) -> TypeId {
        let class_name = self.class_name(class_id);
        let type_name = TypeName::concrete(class_name);
        if let Some(&type_id) = self.global_type_ids_by_name.get(&type_name) {
            type_id
        } else {
            let type_id = self.register_type(type_name.clone(), Type::Object(class_id));
            self.global_type_ids_by_name.insert(type_name, type_id);
            type_id
        }
    }
}

/// # Variable registry
impl Environment {
    pub fn register_var(&mut self, var: Var) -> VarId {
        let id = VarId(self.vars.len() as u32);
        self.vars.push(var);
        id
    }

    pub fn get_var(&self, id: VarId) -> &Var {
        &self.vars[id.0 as usize]
    }
}

/// # Function registry
impl Environment {
    pub fn register_function(&mut self, function: Function) -> FunctionId {
        let id = FunctionId(self.functions.len() as u32);
        self.functions.push(function);
        id
    }

    pub fn get_function(&self, function_id: FunctionId) -> &Function {
        &self.functions[function_id.0 as usize]
    }

    pub fn get_function_ir(&self, function_id: FunctionId) -> Option<&Ir> {
        self.irs_by_function_id.get(&function_id)
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
        self.env.untyped_class_partitions(class_id)
    }

    /// Returns the set of untyped partitions for stealing purposes.
    ///
    /// As the name suggests, you should generally avoid using this. When using this, you're
    /// pledging that you will put back whatever you stole out of the untyped class partitions
    /// in unaltered form after you're done with it.
    pub fn untyped_class_partitions_for_theft(
        &mut self,
        class_id: ClassId,
    ) -> Option<&mut [UntypedClassPartition]> {
        _ = self.untyped_class_partitions(class_id);
        self.env
            .untyped_class_partitions
            .get_mut(&class_id)
            .and_then(|x| x.as_mut())
            .map(|x| x.as_mut_slice())
    }
}

impl Environment {
    fn register_magic_type(&mut self, name: &str, ty: Type) {
        let type_id = self.register_type(TypeName::concrete(name), ty);
        self.global_type_ids_by_name
            .insert(TypeName::concrete(name), type_id);
    }

    fn register_fundamental_types(&mut self) {
        // NOTE: Order matters here! The TypeIds and ClassIds must match exactly those defined
        // in the impls below.
        self.register_magic_type("error type", Type::Error);

        self.register_magic_type("Void", Type::Void);

        self.register_magic_type("Bool", Type::Primitive(Primitive::Bool));
        self.register_magic_type("Byte", Type::Primitive(Primitive::Byte));
        self.register_magic_type("Int", Type::Primitive(Primitive::Int));
        self.register_magic_type("Float", Type::Primitive(Primitive::Float));
        self.register_magic_type("String", Type::Primitive(Primitive::String));
        self.register_magic_type("Name", Type::Primitive(Primitive::Name));

        let object_class = self.get_or_create_class("Object");
        self.register_magic_type("Object", Type::Object(object_class));
    }
}

impl TypeId {
    pub const ERROR: Self = Self(0);
    pub const VOID: Self = Self(1);
    pub const BOOL: Self = Self(2);
    pub const BYTE: Self = Self(3);
    pub const INT: Self = Self(4);
    pub const FLOAT: Self = Self(5);
    pub const STRING: Self = Self(6);
    pub const NAME: Self = Self(7);
    pub const OBJECT: Self = Self(8);
}

impl ClassId {
    pub const OBJECT: Self = Self(0);
}

impl Primitive {
    pub fn id(&self) -> TypeId {
        match self {
            Primitive::Bool => TypeId::BOOL,
            Primitive::Byte => TypeId::BYTE,
            Primitive::Int => TypeId::INT,
            Primitive::Float => TypeId::FLOAT,
            Primitive::String => TypeId::STRING,
            Primitive::Name => TypeId::NAME,
        }
    }
}

/// # Memoized type lookups
impl<'a> Compiler<'a> {
    pub fn type_id(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
    ) -> TypeId {
        let (_source, type_id) = self.type_id_with_source(source_file_id, scope, ty);
        type_id
    }

    pub fn type_id_with_source(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
    ) -> (TypeSource, TypeId) {
        let type_name = TypeName::from_cst(self.sources, source_file_id, ty);
        if let Some(&type_id) = self
            .env
            .scoped_type_ids_by_name
            .get(&(scope, type_name.clone()))
        {
            (TypeSource::Scoped, type_id)
        } else if let Some(&type_id) = self.env.global_type_ids_by_name.get(&type_name) {
            (TypeSource::Global, type_id)
        } else {
            #[allow(deprecated)]
            let (source, type_id) = self.find_type_id(source_file_id, scope, ty);
            // Only cache the result if the type is correct; in case of erroneous type references
            // we don't want to stop emitting errors at the first one.
            if type_id != TypeId::ERROR {
                match source {
                    TypeSource::Global => {
                        self.env.global_type_ids_by_name.insert(type_name, type_id);
                    }
                    TypeSource::Scoped => {
                        self.env
                            .scoped_type_ids_by_name
                            .insert((scope, type_name), type_id);
                    }
                }
            }
            (source, type_id)
        }
    }
}

/// # Memoized function analysis
impl<'a> Compiler<'a> {
    pub fn function_ir(&mut self, function_id: FunctionId) -> &Ir {
        if self.env.get_function_ir(function_id).is_none() {
            let ir = self.analyze_function_body(function_id);
            self.env.irs_by_function_id.insert(function_id, ir);
        }
        self.env.get_function_ir(function_id).unwrap()
    }
}
