use bitflags::bitflags;

mod namespace;
mod var;

use muscript_syntax::token::Ident;
pub use namespace::*;
pub use var::*;

#[derive(Debug, Clone)]
pub struct ClassSpecifiers {
    pub flags: ClassFlags,
    pub auto_expand_categories: Vec<Ident>,
    pub class_group: Vec<Ident>,
    pub config: Option<Ident>,
    // dependson is omitted because MuScript handles compilation order properly.
    pub dont_sort_categories: Vec<Ident>,
    pub hide_categories: Vec<Ident>,
    pub implements: Vec<Ident>,
    // inherits and native are omitted because we don't support emitting C++.
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ClassFlags: u16 {
        const ABSTRACT = 0x1;
        const ALWAYS_LOADED = 0x2;
        const COLLAPSE_CATEGORIES = 0x4;
        const DEPRECATED = 0x8;
        const DONT_COLLAPSE_CATEGORIES = 0x10;
        const EDIT_INLINE_NEW = 0x20;
        const FORCE_SCRIPT_ORDER = 0x40;
        const HIDE_DROPDOWN = 0x80;
        const ITERATION_OPTIMIZED = 0x100;
        const NEVER_COOK = 0x200;
        const NO_EXPORT = 0x400;
        const NOT_PLACEABLE = 0x800;
        const PER_OBJECT_CONFIG = 0x1000;
        const PLACEABLE = 0x2000;
        const TRANSIENT = 0x4000;
    }
}
