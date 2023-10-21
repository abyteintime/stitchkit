use muscript_foundation::errors::{Diagnostic, Label};
use muscript_lexer::{sources::LexedSources, token_stream::TokenStream};
use muscript_syntax_derive::Spanned;

use crate::{
    cst::{Extends, KSimulated},
    list::{SeparatedListDiagnostics, TerminatedListErrorKind},
    token::{AnyToken, Ident, LeftBrace, RightBrace, Semi},
    Parse, ParseError, Parser, PredictiveParse,
};

use super::{Item, VarEditor};

keyword! {
    KAuto = "auto",
    KState = "state",
    KIgnores = "ignores",
}

#[derive(Debug, Clone, Spanned)]
pub struct ItemState {
    pub simulated: Option<KSimulated>,
    pub auto: Option<KAuto>,
    pub state: KState,
    pub editor: Option<VarEditor>,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub open: LeftBrace,
    pub ignores: Option<Ignores>,
    pub items: Vec<Item>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Ignores {
    pub ignores: KIgnores,
    pub events: Vec<Ident>,
    pub semi: Semi,
}

impl Parse for ItemState {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let auto = parser.parse()?;
        let state = parser.parse()?;
        let editor = parser.parse()?;
        let name = parser.parse()?;
        let extends = parser.parse()?;
        let open: LeftBrace = parser.parse()?;
        let ignores = parser.parse()?;
        let (items, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error("missing `}` to close state body")
                        .with_label(Label::primary(&open, "this is where the state body begins")),
                )
            }
            error.parse
        })?;
        Ok(Self {
            simulated: None, // filled in during analysis - partitioning
            auto,
            state,
            editor,
            name,
            extends,
            open,
            ignores,
            items,
            close,
        })
    }
}

impl PredictiveParse for ItemState {
    #[allow(deprecated)]
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool {
        KState::started_by(token, sources) || KAuto::started_by(token, sources)
    }
}

impl Parse for Ignores {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let ignores = parser.parse()?;
        let (events, semi) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &ignores,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `;` after `ignores` item",
                    missing_right_label: "this `ignores` does not have a `;`",
                    missing_comma: "`,` or `;` expected after event name in `ignores`",
                    missing_comma_open: "in this `ignores`",
                    missing_comma_token:
                        "this was expected to continue or end the list of events to be ignored",
                    missing_comma_note: "note: ignored events are separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            ignores,
            events,
            semi,
        })
    }
}
