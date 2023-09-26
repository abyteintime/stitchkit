use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, ReplacementSuggestion},
    ident::CaseInsensitive,
    source::SourceFileId,
    span::Spanned,
};
use muscript_syntax::{cst, lexis::token::Ident};
use tracing::{trace, trace_span};

use crate::{
    partition::{TypeCst, UntypedClassPartition},
    ClassId, Compiler, TypeId,
};

use super::{Primitive, Type, TypeName};

/// The source from which a type ID was generated.
/// This must be `Scoped` if a type ID refers to any types that were found in a class scope.
pub enum TypeSource {
    Global,
    Scoped,
}

const ERROR_RESULT: (TypeSource, TypeId) = (TypeSource::Global, TypeId::ERROR);

impl<'a> Compiler<'a> {
    #[deprecated = "use the memoized version [`Compiler::type_id`]"]
    pub(crate) fn find_type_id(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
    ) -> (TypeSource, TypeId) {
        match &ty.path.components[..] {
            [] => unreachable!("paths must have at least one component"),
            [type_name_ident] => {
                let type_name = self.sources.source(type_name_ident);

                if let Some(type_id) =
                    self.find_type_in_current_scope(source_file_id, scope, ty, *type_name_ident)
                {
                    return (TypeSource::Scoped, type_id);
                } else if let Some(type_id) =
                    self.find_global_type(source_file_id, ty, *type_name_ident)
                {
                    return (TypeSource::Global, type_id);
                } else if let Some(type_id) = Primitive::from_name(type_name) {
                    self.expect_no_generics(source_file_id, ty);
                    return (TypeSource::Global, type_id);
                } else if type_name.eq_ignore_ascii_case("Array") {
                    return self.array_type(source_file_id, scope, ty);
                } else if type_name.eq_ignore_ascii_case("Class") {
                    return (
                        TypeSource::Global,
                        self.class_type(source_file_id, scope, ty),
                    );
                }
            }
            [_class_name, _type_name] => {
                self.env.emit(
                    Diagnostic::bug(
                        "referring to types declared in a different class's scope is not yet implemented"
                    )
                    .with_label(Label::primary(&ty.path, ""))
                );
                return ERROR_RESULT;
            }
            _ => (),
        }

        // TODO: Emit a more helpful diagnostic pointing out possible typos.
        self.env.emit(
            Diagnostic::error(format!(
                "cannot find type `{}` in this scope",
                ty.path.pretty_print(self.sources)
            ))
            .with_label(Label::primary(&ty.path, "")),
        );
        ERROR_RESULT
    }

    fn expect_no_generics(&mut self, source_file_id: SourceFileId, ty: &cst::Type) {
        if let Some(generic) = &ty.generic {
            self.env.emit(
                Diagnostic::error("use of generic arguments on non-generic type")
                    .with_label(Label::primary(generic, ""))
                    .with_label(Label::secondary(&ty.path, "this type is not generic"))
                    .with_note(
                        "note: generics may only be used on built-in types `Class` and `Array`",
                    ),
            );
        }
    }

    fn array_type(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
    ) -> (TypeSource, TypeId) {
        // Array types are a little bit hardcoded at the moment but we have to treat them as magic
        // because they are special from the VM's perspective.
        if let Some(generic) = &ty.generic {
            if let [inner] = &generic.args[..] {
                let (source, inner_id) = self.type_id_with_source(source_file_id, scope, inner);
                (
                    source,
                    self.env.register_type(
                        TypeName::generic("Array", vec![self.env.type_name(inner_id).clone()]),
                        Type::Array(inner_id),
                    ),
                )
            } else {
                self.env.emit(
                    Diagnostic::error(format!(
                        "`Array` expects a single generic argument `<T>`, but got {}",
                        generic.args.len()
                    ))
                    .with_label(Label::primary(generic, "")),
                );
                ERROR_RESULT
            }
        } else {
            self.env.emit(
                Diagnostic::error("`Array` expects one generic argument `<T>`")
                    .with_label(Label::primary(&ty.path, ""))
                    .with_note((
                        "help: try giving the array a type of element to store",
                        self.sources.replacement_suggestion(ty, "Array<Int>"),
                    )),
            );
            ERROR_RESULT
        }
    }

    fn class_type(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
    ) -> TypeId {
        let (super_class_id, super_type_id) = if let Some(generic) = &ty.generic {
            match &generic.args[..] {
                [] => (ClassId::OBJECT, TypeId::OBJECT),
                [inner] => {
                    let inner_id = self.type_id(source_file_id, scope, inner);
                    if let &Type::Object(class_id) = self.env.get_type(inner_id) {
                        (class_id, inner_id)
                    } else {
                        self.env.emit(Diagnostic::error(format!(
                            "`{}` is not a class",
                            self.env.type_name(inner_id)
                        )));
                        return TypeId::ERROR;
                    }
                }
                _ => {
                    self.env.emit(
                        Diagnostic::error(format!(
                            "`Class` expects a single generic argument `<T>`, but got {}",
                            generic.args.len()
                        ))
                        .with_label(Label::primary(generic, "")),
                    );
                    return TypeId::ERROR;
                }
            }
        } else {
            (ClassId::OBJECT, TypeId::OBJECT)
        };

        self.env.register_type(
            TypeName::generic("Class", vec![self.env.type_name(super_type_id).clone()]),
            Type::Class(super_class_id),
        )
    }

    fn generics_not_allowed(
        &mut self,
        source_file_id: SourceFileId,
        ty: &cst::Type,
        generic: &cst::Generic,
        type_name: &str,
    ) {
        self.env.emit(
            Diagnostic::error("only `Array` and `Class` may use generics")
                .with_label(Label::primary(generic, ""))
                .with_note((
                    "help: remove the generic parameters",
                    self.sources
                        .replacement_suggestion(ty, type_name.to_string()),
                )),
        )
    }

    fn find_global_type(
        &mut self,
        source_file_id: SourceFileId,
        ty: &cst::Type,
        type_name_ident: Ident,
    ) -> Option<TypeId> {
        let type_name = self.sources.source(&type_name_ident);

        if self.input.class_exists(type_name) {
            // NOTE: Do not process the class here anyhow! Only create a type for it.
            let class_id = self.env.get_or_create_class(type_name);
            let type_id = self
                .env
                .register_type(TypeName::concrete(type_name), Type::Object(class_id));
            if let Some(generic) = &ty.generic {
                self.generics_not_allowed(source_file_id, ty, generic, type_name);
            }
            Some(type_id)
        } else {
            None
        }
    }

    fn find_type_in_current_scope(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
        type_name_ident: Ident,
    ) -> Option<TypeId> {
        let type_name = self.sources.source(&type_name_ident);
        trace!(scope = self.env.class_name(scope), %type_name, "find_type_in_current_scope");

        if let Some(partitions) = self.untyped_class_partitions(scope) {
            let type_impl = partitions
                .iter()
                .find_map(|partition| partition.types.get(CaseInsensitive::new_ref(type_name)))
                .map(|type_cst| match type_cst {
                    TypeCst::Struct(_) => Type::Struct { outer: scope },
                    TypeCst::Enum(_) => Type::Enum { outer: scope },
                });
            if let Some(type_impl) = type_impl {
                if let Some(generic) = &ty.generic {
                    let type_name = self.sources.source(&type_name_ident);
                    self.generics_not_allowed(source_file_id, ty, generic, type_name);
                }
                return Some(
                    self.env
                        .register_type(TypeName::concrete(type_name), type_impl),
                );
            }
        }

        if let Some(next_scope) = self.super_class_id(scope) {
            trace!(?scope, ?next_scope, "Recurring upwards to super scope");
            // NOTE: We need to do a tail call here back to type_id, for memoization to work
            // correctly. Given these two classes:
            //
            //     class Foo extends Object;
            //     struct Example {}
            //     var Example Boo;
            //
            //     class Bar extends Foo;
            //     var Example Far;
            //
            // We do not want to reregister `Example` both for `Foo` and `Bar`, rather we want to
            // reuse an already existing TypeId.
            return Some(self.type_id(source_file_id, next_scope, ty));
        }
        None
    }

    /// Obtain the super class of the given class, or `None` if it has no super class declared.
    pub fn super_class_id(&mut self, class_id: ClassId) -> Option<ClassId> {
        let _span = trace_span!("super_class_type", ?class_id).entered();
        if let Some(partitions) = self.untyped_class_partitions(class_id) {
            // TODO: This can get weird if there is more than one partition declaring an `extends`
            // clause, and the `extends` clauses are different.
            if let Some(partition) = partitions
                .iter()
                .find(|partition| partition.extends.is_some())
            {
                let &UntypedClassPartition {
                    source_file_id,
                    extends,
                    ..
                } = partition;
                let class_name = self.sources.source(&extends.unwrap());
                return self.lookup_class(source_file_id, class_name, extends.unwrap().span());
            }
        }
        None
    }
}
