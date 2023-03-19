use crate::TypeId;

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(Primitive),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Byte,
    Int,
    Float,
    String,
    Name,
}

impl Primitive {
    pub fn from_name(name: &str) -> Option<TypeId> {
        Some(match name {
            _ if name.eq_ignore_ascii_case("Byte") => Primitive::Byte.id(),
            _ if name.eq_ignore_ascii_case("Int") => Primitive::Int.id(),
            _ if name.eq_ignore_ascii_case("Float") => Primitive::Float.id(),
            _ if name.eq_ignore_ascii_case("String") => Primitive::String.id(),
            _ if name.eq_ignore_ascii_case("Name") => Primitive::Name.id(),
            _ => return None,
        })
    }
}
