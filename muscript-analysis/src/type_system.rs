pub mod lookup;
mod subtyping;

use std::fmt;

use muscript_foundation::ident::CaseInsensitive;
use muscript_lexer::sources::LexedSources;
use muscript_syntax::cst;

use crate::{ClassId, TypeId};

#[derive(Debug, Clone)]
pub enum Type {
    Error,
    /// `Void`; the lack of a meaningful value.
    ///
    /// `Void` is not treated as a primitive since it does not have a corresponding literal
    /// or value.
    Void,
    Primitive(Primitive),
    /// `Array<T>`
    Array(TypeId),
    /// `T`
    Object(ClassId),
    /// `class<T>`
    Class(ClassId),
    /// Structs and enums don't actually store any metadata here, since they're processed already
    /// as part of the class partition. You can use type_name to retrieve their CST, fields, etc.
    /// from their outer class.
    Struct {
        outer: ClassId,
    },
    Enum {
        outer: ClassId,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeName {
    pub name: CaseInsensitive<String>,
    pub generic_arguments: Vec<TypeName>,
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

impl TypeName {
    pub fn concrete(name: impl Into<String>) -> Self {
        Self {
            name: CaseInsensitive::new(name.into()),
            generic_arguments: vec![],
        }
    }

    pub fn generic(name: impl Into<String>, args: Vec<Self>) -> Self {
        Self {
            generic_arguments: args,
            ..Self::concrete(name)
        }
    }

    pub fn from_cst(sources: &LexedSources<'_>, ty: &cst::Type) -> Self {
        Self {
            name: CaseInsensitive::new(sources.source(&ty.path).to_owned()),
            generic_arguments: ty
                .generic
                .iter()
                .flat_map(|generic| generic.args.iter().map(|ty| Self::from_cst(sources, ty)))
                .collect(),
        }
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        if !self.generic_arguments.is_empty() {
            f.write_str("<")?;
            for (i, argument) in self.generic_arguments.iter().enumerate() {
                if i != 0 {
                    f.write_str(", ")?;
                }
                fmt::Display::fmt(argument, f)?;
            }
            f.write_str(">")?;
        }
        Ok(())
    }
}

impl From<&str> for TypeName {
    fn from(value: &str) -> Self {
        Self::concrete(value)
    }
}
