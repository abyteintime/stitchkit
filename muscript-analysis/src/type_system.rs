use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::{SourceFileId, Spanned},
};
use muscript_syntax::cst;

use crate::{Compiler, TypeId};

#[derive(Debug, Clone)]
pub enum Type {
    Error,
    Primitive(Primitive),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    Byte,
    Int,
    Float,
    String,
    Name,
}

impl Primitive {
    pub fn from_name(name: &str) -> Option<TypeId> {
        Some(match name {
            _ if name.eq_ignore_ascii_case("Bool") => Primitive::Bool.id(),
            _ if name.eq_ignore_ascii_case("Byte") => Primitive::Byte.id(),
            _ if name.eq_ignore_ascii_case("Int") => Primitive::Int.id(),
            _ if name.eq_ignore_ascii_case("Float") => Primitive::Float.id(),
            _ if name.eq_ignore_ascii_case("String") => Primitive::String.id(),
            _ if name.eq_ignore_ascii_case("Name") => Primitive::Name.id(),
            _ => return None,
        })
    }
}

impl<'a> Compiler<'a> {
    pub fn type_id(&mut self, source_file_id: SourceFileId, ty: &cst::Type) -> TypeId {
        match &ty.path.components[..] {
            [] => unreachable!("paths must have at least one component"),
            [type_name_ident] => {
                let type_name = self.sources.span(source_file_id, type_name_ident);
                if let Some(type_id) = Primitive::from_name(type_name) {
                    self.expect_no_generics(source_file_id, ty);
                    return type_id;
                }
            }
            [_class_name, _type_name] => {
                self.env.emit(
                    Diagnostic::bug(
                        source_file_id,
                        "referring to types declared in a different class's scope is not yet implemented"
                    ).with_label(Label::primary(ty.path.span(), "")),
                );
                return TypeId::ERROR;
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
        TypeId::ERROR
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
}
