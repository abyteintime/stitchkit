use muscript_syntax_derive::Spanned;

use crate::{Braces, LazyBlock, Parse, PredictiveParse};

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct CppBlob {
    pub blob: LazyBlock<Braces>,
}
