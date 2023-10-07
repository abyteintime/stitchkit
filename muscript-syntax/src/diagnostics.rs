pub mod labels {
    use muscript_foundation::errors::Label;
    use muscript_lexer::{
        sources::LexedSources,
        token::{Token, TokenSpan},
    };

    pub fn invalid_identifier(span: TokenSpan, sources: &LexedSources<'_>) -> Label<Token> {
        Label::primary(
            &span,
            format!("`{}` is not a valid identifier", sources.source(&span)),
        )
    }
}

pub mod notes {
    pub const IDENTIFIER_CHARS: &str = "note: identifiers are made up of characters a-z, A-Z, 0-9 and _, and must not start with a digit";

    pub const PARSER_BUG: &str = "note: if you're seeing this, there's likely a problem with the parser.\n      please report an issue at https://github.com/abyteintime/stitchkit";
}

pub mod sets {
    use crate::list::SeparatedListDiagnostics;

    pub static VARIABLES: SeparatedListDiagnostics = SeparatedListDiagnostics {
        missing_right: "missing `;` to end variable declaration",
        missing_right_label: "this variable declaration does not have a `;`",
        missing_comma: "`,` or `;` expected after variable name",
        missing_comma_open: "this is the variable declaration",
        missing_comma_token: "this was expected to continue or end the variable declaration",
        missing_comma_note:
            "note: multiple variable names in one `var` must be separated by commas `,`",
    };
}
