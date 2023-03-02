pub mod labels {
    use muscript_foundation::{errors::Label, source::Span};

    pub fn invalid_identifier(span: Span, input: &str) -> Label {
        Label::primary(
            span,
            format!("`{}` is not a valid identifier", span.get_input(input)),
        )
    }
}

pub mod notes {
    pub const IDENTIFIER_CHARS: &str = "note: identifiers are made up of characters a-z, A-Z, 0-9 and _, and must not start with a digit";

    pub const PARSER_BUG: &str = "note: if you're seeing this, there's likely a problem with the parser.\n      please report an issue at https://github.com/abyteintime/stitchkit";
}
