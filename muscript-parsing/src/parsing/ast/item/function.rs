use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::{Parse, PredictiveParse};

use crate::{
    lexis::{
        token::{Ident, LeftBrace, LeftParen, RightBrace, RightParen},
        TokenStream,
    },
    parsing::{
        ast::{Stmt, Type},
        diagnostics::{labels, notes},
        list::{DelimitedListDiagnostics, TerminatedListErrorKind},
        Parse, ParseError, Parser,
    },
};

keyword!(KFunction = "function");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemFunction {
    pub function: KFunction,
    pub name: Ident,
    pub params: Params,
    pub body: Body,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Params {
    pub open: LeftParen,
    pub params: Vec<Param>,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Param {
    pub ty: Type,
    pub name: Ident,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Body {
    pub open: LeftBrace,
    pub stmts: Vec<Stmt>,
    pub close: RightBrace,
}

impl Parse for ItemFunction {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            function: parser.parse()?,
            name: parser.parse_with_error(|parser, span| {
                Diagnostic::error(parser.file, "function name expected")
                    .with_label(labels::invalid_identifier(span, parser.input))
                    .with_note(notes::IDENTIFIER_CHARS)
            })?,
            params: parser.parse()?,
            body: parser.parse()?,
        })
    }
}

impl Parse for Params {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let (open, params, close) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(
                error,
                DelimitedListDiagnostics {
                    missing_left: "function parameters `(int x, int y, ..)` expected",
                    missing_left_label: "function parameters expected here",
                    missing_right: "missing `)` to close function parameter list",
                    missing_comma: "`,` or `)` expected after parameter",
                    missing_comma_token:
                        "this was expected to continue or close the parameter list",
                    missing_comma_open: "the parameter list starts here",
                    missing_comma_note: "note: function parameters must be separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            open,
            params,
            close,
        })
    }
}

impl Parse for Body {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "function body `{ .. }` expected")
                .with_label(Label::primary(span, "`{` expected here"))
        })?;
        let (stmts, close) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `}` to close function body")
                        .with_label(Label::primary(open.span, "this is where the body begins"))
                        .with_note(notes::PARSER_BUG),
                ),
            }
            error.parse
        })?;
        Ok(Self { open, stmts, close })
    }
}
