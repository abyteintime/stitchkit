use muscript_foundation::span::Spanned;
use muscript_syntax_derive::Spanned;

use crate::lexis::token::{Token, TokenSpan};

use super::{EnumDef, ItemConst, ItemEnum, ItemFunction, ItemState, ItemStruct, StructDef};

/// Item names can be made out of multiple tokens (as is the case with eg. function names, which
/// can be `+=` - two tokens.)
#[derive(Debug, Clone, Copy, Spanned)]
pub struct ItemName {
    pub span: TokenSpan,
}

impl ItemName {
    pub fn from_spanned(spanned: &impl Spanned<Token>) -> Self {
        Self {
            span: spanned.span(),
        }
    }
}

pub trait NamedItem {
    fn name(&self) -> ItemName;
}

// NOTE: ItemVar does not implement NamedItem because it can declare multiple variables.
// See muscript-analysis which defines an ItemSingleVar, which does not have this problem.

macro_rules! impl_named_item {
    ($T:ty => |$this:tt| $field:expr) => {
        impl NamedItem for $T {
            fn name(&self) -> ItemName {
                let $this = self;
                $field
            }
        }
    };
}

impl_named_item!(ItemConst => |item| ItemName { span: item.name.span() });
impl_named_item!(ItemFunction => |item| item.name);
impl_named_item!(ItemStruct => |item| ItemName { span: item.def.name.span() });
impl_named_item!(StructDef => |def| ItemName { span: def.name.span() });
impl_named_item!(ItemEnum => |item| ItemName { span: item.def.name.span() });
impl_named_item!(EnumDef => |def| ItemName { span: def.name.span() });
impl_named_item!(ItemState => |item| ItemName { span: item.name.span() });

impl<T> NamedItem for Box<T>
where
    T: NamedItem,
{
    fn name(&self) -> ItemName {
        (**self).name()
    }
}
