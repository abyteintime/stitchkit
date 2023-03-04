use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    ast::{Stmt, Type},
    diagnostics::{labels, notes},
    lexis::{
        token::{Ident, LeftBrace, LeftParen, RightBrace, RightParen, TokenKind},
        TokenStream,
    },
    list::{DelimitedListDiagnostics, TerminatedListErrorKind},
    Parse, ParseError, Parser, PredictiveParse,
};

keyword!(KFunction = "function");

#[derive(Debug, Clone, PredictiveParse)]
pub struct ItemFunction {
    pub function: KFunction,
    pub return_ty: Option<Type>,
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

impl ItemFunction {
    fn parse_name(parser: &mut Parser<'_, impl TokenStream>) -> Result<Ident, ParseError> {
        parser.parse_with_error::<Ident>(|parser, span| {
            Diagnostic::error(parser.file, "function name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })
    }
}

impl Parse for ItemFunction {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let function = parser.parse()?;

        // NOTE: We need to do a little dance to parse return types (though it's still 100% possible
        // to do predictively, which is very nice.) This would've been easier if return types
        // weren't optional, but alas.
        let name_or_type = Self::parse_name(parser)?;
        let (return_ty, name) = if parser.peek_token()?.kind == TokenKind::LeftParen {
            (None, name_or_type)
        } else {
            let generic = parser.parse()?;
            (
                Some(Type {
                    name: name_or_type,
                    generic,
                }),
                Self::parse_name(parser)?,
            )
        };

        let params = parser.parse()?;
        let body = parser.parse()?;
        Ok(Self {
            function,
            return_ty,
            name,
            params,
            body,
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
