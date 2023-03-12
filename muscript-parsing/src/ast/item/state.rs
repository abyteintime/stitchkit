use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{Ident, LeftBrace, RightBrace, Semi, Token},
    list::{SeparatedListDiagnostics, TerminatedListErrorKind},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{Item, VarEditor};

keyword! {
    KAuto = "auto",
    KState = "state",
    KIgnores = "ignores",
}

#[derive(Debug, Clone)]
pub struct ItemState {
    pub auto: Option<KAuto>,
    pub state: KState,
    pub editor: Option<VarEditor>,
    pub name: Ident,
    pub open: LeftBrace,
    pub ignores: Option<Ignores>,
    pub items: Vec<Item>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Ignores {
    pub ignores: KIgnores,
    pub events: Vec<Ident>,
    pub semi: Semi,
}

impl Parse for ItemState {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let auto = parser.parse()?;
        let state = parser.parse()?;
        let editor = parser.parse()?;
        let name = parser.parse()?;
        let open: LeftBrace = parser.parse()?;
        let ignores = parser.parse()?;
        let (items, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `}` to close state body").with_label(
                        Label::primary(open.span, "this is where the state body begins"),
                    ),
                )
            }
            error.parse
        })?;
        Ok(Self {
            auto,
            state,
            editor,
            name,
            open,
            ignores,
            items,
            close,
        })
    }
}

impl PredictiveParse for ItemState {
    fn started_by(token: &Token, input: &str) -> bool {
        KState::started_by(token, input) || KAuto::started_by(token, input)
    }
}

impl Parse for Ignores {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
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
