use crate::lexis::token::Ident;

use super::{EnumDef, ItemConst, ItemEnum, ItemFunction, ItemState, ItemStruct, StructDef};

pub trait NamedItem {
    fn name(&self) -> Ident;
}

// NOTE: ItemVar does not implement NamedItem because it can declare multiple variables.
// See muscript-analysis which defines an ItemSingleVar, which does not have this problem.

macro_rules! impl_named_item {
    ($T:ty => |$this:tt| $field:expr) => {
        impl NamedItem for $T {
            fn name(&self) -> Ident {
                let $this = self;
                $field
            }
        }
    };
}

impl_named_item!(ItemConst => |item| item.name);
impl_named_item!(ItemFunction => |item| item.name);
impl_named_item!(ItemStruct => |item| item.def.name);
impl_named_item!(StructDef => |def| def.name);
impl_named_item!(ItemEnum => |item| item.def.name);
impl_named_item!(EnumDef => |def| def.name);
impl_named_item!(ItemState => |item| item.name);

impl<T> NamedItem for Box<T>
where
    T: NamedItem,
{
    fn name(&self) -> Ident {
        (**self).name()
    }
}
