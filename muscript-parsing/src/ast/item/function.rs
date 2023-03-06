use indoc::indoc;
use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    ast::{
        Expr, IntLit, KCoerce, KConst, KFinal, KNative, KOptional, KOut, KSimulated, KSkip,
        KStatic, Stmt, Type,
    },
    diagnostics::{labels, notes},
    lexis::token::{
        Assign, Ident, LeftBrace, LeftParen, RightBrace, RightParen, Semi, Token, TokenKind,
    },
    list::{DelimitedListDiagnostics, TerminatedListErrorKind},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

keyword! {
    KFunction = "function",
    KEvent = "event",
    KOperator = "operator",
    KPreOperator = "preoperator",
    KPostOperator = "postoperator",
}

#[derive(Debug, Clone)]
pub struct ItemFunction {
    pub specifiers: Vec<FunctionSpecifier>,
    pub function: FunctionKind,
    pub return_ty: Option<Type>,
    pub name: Ident,
    pub params: Params,
    pub body: Body,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "function_specifier_error")]
pub enum FunctionSpecifier {
    Final(KFinal),
    Native(KNative, Option<ParenInt>),
    Simulated(KSimulated),
    Static(KStatic),
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ParenInt {
    pub left: LeftParen,
    pub number: IntLit,
    pub right: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "kind_error")]
pub enum FunctionKind {
    Function(KFunction),
    Event(KEvent),
    Operator(KOperator, ParenInt),
    PreOperator(KPreOperator),
    PostOperator(KPostOperator),
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Params {
    pub open: LeftParen,
    pub params: Vec<Param>,
    pub close: RightParen,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub specifiers: Vec<ParamSpecifier>,
    pub ty: Type,
    pub name: Ident,
    pub default: Option<ParamDefault>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "param_specifier_error")]
pub enum ParamSpecifier {
    Coerce(KCoerce),
    Const(KConst),
    Optional(KOptional),
    Out(KOut),
    Skip(KSkip),
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ParamDefault {
    pub equals: Assign,
    pub value: Expr,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "body_error")]
pub enum Body {
    Stub(Semi),
    Impl(Impl),
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Impl {
    pub open: LeftBrace,
    pub stmts: Vec<Stmt>,
    pub close: RightBrace,
}

impl ItemFunction {
    fn parse_name(parser: &mut Parser<'_, impl ParseStream>) -> Result<Ident, ParseError> {
        parser.parse_with_error::<Ident>(|parser, span| {
            Diagnostic::error(parser.file, "function name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })
    }
}

impl Parse for ItemFunction {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let specifiers = parser.parse_greedy_list()?;
        let function = parser.parse()?;

        let (return_ty, name) = match &function {
            FunctionKind::Function(_) | FunctionKind::Event(_) => {
                // NOTE: We need to do a little dance to parse return types (though it's still 100%
                // possible to do predictively, which is very nice.) This would've been easier if
                // return types weren't optional, but alas.
                let name_or_type = Self::parse_name(parser)?;
                if parser.peek_token()?.kind == TokenKind::LeftParen {
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
                }
            }
            FunctionKind::Operator(_, _)
            | FunctionKind::PreOperator(_)
            | FunctionKind::PostOperator(_) => {
                // Operators need special care because they always have a return type and, more
                // importantly, the name is not an identifier but an operator.
                // For practical reasons we still make it pose as an identifier, but the lexer
                // doesn't see it exactly that way.
                let return_ty = parser.parse()?;
                let operator = parser.next_token()?;
                if !operator.kind.is_overloadable_operator() {
                    parser.emit_diagnostic(
                        Diagnostic::error(
                            parser.file,
                            format!(
                                "`{}` is not an overloadable operator",
                                operator.span.get_input(parser.input)
                            ),
                        )
                        .with_label(Label::primary(operator.span, "operator expected here"))
                        .with_note(indoc!(
                            r#"note: overloadable operators include:
                                     `+` `-` `*` `/` `%` `**`
                                     `$` `@`
                                     `<<` `>>` `>>>` `~` `&` `|` `^`
                                     `!` `==` `!=` `~=` `<` `>` `<=` `>=`
                                     `&&` `||` `^^`
                                     `++` `--`
                                     and identifiers
                            "#
                        )),
                    )
                    // NOTE: Don't return a parse error here, just continue on parsing to maybe
                    // find another error.
                }
                let assign = parser.parse::<Option<Assign>>();
                let span = if let Ok(Some(assign)) = assign {
                    operator.span.join(&assign.span)
                } else {
                    operator.span
                };
                (
                    Some(Type {
                        // Ugly hack to work around the fact that `int <` is a valid operator
                        // overload. FML.
                        name: return_ty,
                        generic: None,
                    }),
                    Ident { span },
                )
            }
        };

        let params = parser.parse()?;
        let body = parser.parse()?;
        Ok(Self {
            specifiers,
            function,
            return_ty,
            name,
            params,
            body,
        })
    }
}

impl PredictiveParse for ItemFunction {
    fn started_by(token: &Token, input: &str) -> bool {
        // Kind of sub-optimal that we have to check here each and every single identifier.
        KFunction::started_by(token, input) || FunctionSpecifier::started_by(token, input)
    }
}

impl Parse for Params {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open: LeftParen = parser.parse()?;
        let (params, close) = parser.parse_delimited_list().map_err(|error| {
            parser.emit_delimited_list_diagnostic(
                &open,
                error,
                DelimitedListDiagnostics {
                    missing_right: "missing `)` to close function parameter list",
                    missing_right_label: "this `(` does not have a matching `)`",
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

impl Parse for Param {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let specifiers = parser.parse_greedy_list()?;
        let ty = parser.parse()?;
        let name = parser.parse()?;
        let default = parser.parse()?;
        Ok(Self {
            specifiers,
            ty,
            name,
            default,
        })
    }
}

impl PredictiveParse for Param {
    fn started_by(token: &Token, input: &str) -> bool {
        Ident::started_by(token, input)
    }
}

impl Parse for Impl {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
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

fn function_specifier_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown function specifier `{}`",
            token.span.get_input(parser.input)
        ),
    )
    .with_label(Label::primary(
        token.span,
        "this specifier is not recognized",
    ))
    .with_note("note: notable function specifiers include `static` and `final`")
}

fn param_specifier_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown parameter specifier `{}`",
            token.span.get_input(parser.input)
        ),
    )
    .with_label(Label::primary(
        token.span,
        "this specifier is not recognized",
    ))
    .with_note("note: notable parameter specifiers include `optional` and `out`")
}

fn kind_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        "`function`, `event`, `preoperator`, or `operator` expected",
    )
    .with_label(Label::primary(
        token.span,
        "this token does not start a function",
    ))
    .with_note("help: maybe you typo'd a specifier?")
}

fn body_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "function body `{ .. }` expected")
    .with_label(Label::primary(token.span, "`{` expected here"))
    .with_note(
        "note: functions can also be stubbed out using `;`, but it's probably not what you want"
    )
}
