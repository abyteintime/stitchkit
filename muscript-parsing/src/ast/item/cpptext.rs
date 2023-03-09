use crate::{ast::CppBlob, Parse, PredictiveParse};

keyword!(KCppText = "cpptext");
keyword!(KStructCppText = "structcpptext");

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemCppText {
    pub cpptext: KCppText,
    pub blob: CppBlob,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemStructCppText {
    pub cpptext: KStructCppText,
    pub blob: CppBlob,
}
