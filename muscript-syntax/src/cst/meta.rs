use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    diagnostics::{labels, notes},
    lexis::token::{Assign, BitOr, Greater, Ident, Less, TokenKind, TokenSpan},
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Meta {
    pub open: Less,
    pub pairs: Vec<MetaValue>,
    pub close: Greater,
}

#[derive(Debug, Clone, Spanned)]
pub enum MetaValue {
    Switch(Ident),
    Pair(Ident, Assign, TokenSpan),
}

impl Parse for Meta {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open = parser.parse()?;
        let (pairs, close) = parser
            .parse_separated_list::<_, _, BitOr>()
            .map_err(|error| {
                parser.emit_separated_list_diagnostic(
                    &open,
                    error,
                    SeparatedListDiagnostics {
                        missing_right: "missing `>` to close metadata list",
                        missing_right_label: "this `<` does not have a matching `>`",
                        missing_comma: "`|` or `>` expected",
                        missing_comma_open: "this is where the metadata list begins",
                        missing_comma_token: "this was expected to continue or close the list",
                        missing_comma_note:
                            "note: unlike most other lists, metadata are separated with pipes `|`",
                    },
                )
            })?;
        Ok(Self { open, pairs, close })
    }
}

#[doc(hidden)]
impl Parse for MetaValue {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let key: Ident = parser.parse_with_error(|parser, span| {
            Diagnostic::error("metadata key expected")
                .with_label(labels::invalid_identifier(span, &parser.sources))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        if let Some(assign) = parser.parse()? {
            let mut value = TokenSpan::Empty;
            loop {
                let token = parser.peek_token();
                match token.kind {
                    TokenKind::BitOr | TokenKind::Greater => break,
                    TokenKind::EndOfFile => {
                        parser.emit_diagnostic(
                            Diagnostic::error(
                                "metadata pair does not have a `|` or `>` that would end it",
                            )
                            .with_label(Label::primary(
                                &key,
                                "this metadatum does not have an end",
                            )),
                        );
                        return Err(parser.make_error(TokenSpan::single(key.id)));
                    }
                    _ => (),
                }
                let token = parser.next_token();
                value = value.join(&TokenSpan::single(token.id));
            }
            Ok(MetaValue::Pair(key, assign, value))
        } else {
            Ok(MetaValue::Switch(key))
        }
    }
}
