use muscript_syntax_derive::Spanned;

use crate::{cst::CppBlob, Parse, PredictiveParse};

keyword!(KCppText = "cpptext");
keyword!(KStructCppText = "structcpptext");

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemCppText {
    pub cpptext: KCppText,
    pub blob: CppBlob,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ItemStructCppText {
    pub cpptext: KStructCppText,
    pub blob: CppBlob,
}
