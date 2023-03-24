use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, ReplacementSuggestion},
    source::{SourceFileId, Spanned},
};
use muscript_syntax::{cst, lexis::token::Ident};

use crate::{partition::UntypedClassPartition, ClassId, Compiler, TypeId};

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
                let type_name = self.sources.span(source_file_id, type_name_ident);
                if let Some(type_id) = Primitive::from_name(type_name) {
                    self.expect_no_generics(source_file_id, ty);
                    return (TypeSource::Global, type_id);
                } else if type_name.eq_ignore_ascii_case("Array") {
                    return self.array_type(source_file_id, scope, ty);
                } else if type_name.eq_ignore_ascii_case("Class") {
                    // Classes are always globally scoped.
                    return (
                        TypeSource::Global,
                        self.class_type(source_file_id, scope, ty),
                    );
                } else if let Some(type_id) =
                    self.find_global_type(source_file_id, ty, *type_name_ident, &ty.generic)
                {
                    return (TypeSource::Global, type_id);
                }
            }
            [_class_name, _type_name] => {
                self.env.emit(
                    Diagnostic::bug(
                        source_file_id,
                        "referring to types declared in a different class's scope is not yet implemented"
                    )
                    .with_label(Label::primary(ty.path.span(), ""))
                );
                return ERROR_RESULT;
            }
            _ => (),
        }

        // TODO: Emit a more helpful diagnostic pointing out possible typos.
        self.env.emit(
            Diagnostic::error(
                source_file_id,
                format!(
                    "cannot find type `{}` in this scope",
                    ty.path.pretty_print(self.sources.source(source_file_id))
                ),
            )
            .with_label(Label::primary(ty.path.span(), "")),
        );
        ERROR_RESULT
    }

    fn expect_no_generics(&mut self, source_file_id: SourceFileId, ty: &cst::Type) {
        if let Some(generic) = &ty.generic {
            self.env.emit(
                Diagnostic::error(
                    source_file_id,
                    "use of generic arguments on non-generic type",
                )
                .with_label(Label::primary(generic.span(), ""))
                .with_label(Label::secondary(ty.path.span(), "this type is not generic"))
                .with_note("note: generics may only be used on built-in types `Class` and `Array`"),
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
                    Diagnostic::error(
                        source_file_id,
                        format!(
                            "`Array` expects a single generic argument `<T>`, but got {}",
                            generic.args.len()
                        ),
                    )
                    .with_label(Label::primary(generic.span(), "")),
                );
                ERROR_RESULT
            }
        } else {
            self.env.emit(
                Diagnostic::error(source_file_id, "`Array` expects one generic argument `<T>`")
                    .with_label(Label::primary(ty.path.span(), ""))
                    .with_note((
                        "help: try giving the array a type of element to store",
                        ReplacementSuggestion {
                            span: ty.span(),
                            // Maybe this suggestion could be a bit more accurate, but the point
                            // is to show you how `Array` is meant to be used.
                            replacement: "Array<Int>".into(),
                        },
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
                        self.env.emit(Diagnostic::error(
                            source_file_id,
                            format!("`{}` is not a class", self.env.type_name(inner_id)),
                        ));
                        return TypeId::ERROR;
                    }
                }
                _ => {
                    self.env.emit(
                        Diagnostic::error(
                            source_file_id,
                            format!(
                                "`Class` expects a single generic argument `<T>`, but got {}",
                                generic.args.len()
                            ),
                        )
                        .with_label(Label::primary(generic.span(), "")),
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
            Diagnostic::error(source_file_id, "only `Array` and `Class` may use generics")
                .with_label(Label::primary(generic.span(), ""))
                .with_note((
                    "help: remove the generic parameters",
                    ReplacementSuggestion {
                        span: ty.span(),
                        replacement: type_name.to_string(),
                    },
                )),
        )
    }

    fn find_global_type(
        &mut self,
        source_file_id: SourceFileId,
        ty: &cst::Type,
        type_name_ident: Ident,
        generic: &Option<cst::Generic>,
    ) -> Option<TypeId> {
        let type_name = self.sources.span(source_file_id, &type_name_ident);

        if let Some(generic) = generic {
            self.generics_not_allowed(source_file_id, ty, generic, type_name);
        }

        if self.input.class_exists(type_name) {
            // NOTE: Do not process the class here anyhow! Only create a type for it.
            let class_id = self.env.get_or_create_class(type_name);
            let type_id = self
                .env
                .register_type(TypeName::concrete(type_name), Type::Object(class_id));
            Some(type_id)
        } else {
            None
        }
    }

    fn find_type_in_scope(
        &mut self,
        source_file_id: SourceFileId,
        scope: ClassId,
        ty: &cst::Type,
        type_name_ident: Ident,
        generic: &Option<cst::Generic>,
    ) -> Option<TypeId> {
        let type_name = self.sources.span(source_file_id, &type_name_ident);

        if let Some(generic) = generic {
            self.generics_not_allowed(source_file_id, ty, generic, type_name);
        }

        let mut current_scope = scope;
        loop {
            if let Some(next_scope) = self.super_class_id(current_scope) {
                current_scope = next_scope;
            } else {
                break;
            }
        }
        None
    }

    /// Obtain the super class of the given class, or `None` if it has no super class declared.
    pub fn super_class_type(&mut self, class_id: ClassId) -> Option<(SourceFileId, Ident, TypeId)> {
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
                let ty = self.type_id(
                    source_file_id,
                    class_id,
                    &cst::Type {
                        specifiers: vec![],
                        path: cst::Path::new(vec![extends.unwrap()]),
                        generic: None,
                        cpptemplate: None,
                    },
                );
                return (ty != TypeId::ERROR).then_some((source_file_id, extends.unwrap(), ty));
            }
        }
        None
    }

    pub fn super_class_id(&mut self, class_id: ClassId) -> Option<ClassId> {
        if let Some((source_file_id, super_class_ident, super_class)) =
            self.super_class_type(class_id)
        {
            if let &Type::Object(class_id) = self.env.get_type(super_class) {
                Some(class_id)
            } else {
                self.env.emit(
                    Diagnostic::error(
                        source_file_id,
                        format!("`{}` is not a class type", self.env.type_name(super_class)),
                    )
                    .with_label(Label::primary(super_class_ident.span, ""))
                    // TODO: Augment this error with the kind of type that was actually provided.
                    .with_note("note: classes can only extend other classes"),
                );
                None
            }
        } else {
            None
        }
    }
}
