use muscript_foundation::source::Span;

use crate::{
    lexis::token::{LeftBrace, RightBrace},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

#[derive(Debug, Clone, PredictiveParse)]
pub struct CppBlob {
    pub open: LeftBrace,
    pub blob: Span,
    pub close: RightBrace,
}

impl Parse for CppBlob {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse()?;
        let blob = parser.tokens.braced_string(open.span).map_err(|error| {
            parser.emit_diagnostic(*error.diagnostic);
            ParseError::new(error.span)
        })?;
        let close = parser.parse()?;
        Ok(Self { open, blob, close })
    }
}
