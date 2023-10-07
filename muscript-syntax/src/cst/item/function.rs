use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, Label},
    span::Spanned,
};
use muscript_lexer::{
    sources::LexedSources,
    token::{Token, TokenKind},
    token_stream::TokenStream,
};
use muscript_syntax_derive::Spanned;

use crate::{
    cst::{Block, Expr, KConst, Path, Type},
    diagnostics::{labels, notes},
    list::SeparatedListDiagnostics,
    token::{AnyToken, Assign, Ident, IntLit, LeftParen, RightParen, Semi},
    Parse, ParseError, Parser, PredictiveParse,
};

use super::{ItemName, VarArray};

#[derive(Debug, Clone, Spanned)]
pub struct ItemFunction {
    pub pre_specifiers: Vec<FunctionSpecifier>,
    pub kind: FunctionKind,
    pub post_specifiers: Vec<FunctionSpecifier>,
    pub return_ty: Option<Type>,
    pub name: ItemName,
    pub params: Params,
    pub kconst: Option<KConst>,
    pub body: Body,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "function_specifier_error")]
pub enum FunctionSpecifier {
    #[parse(keyword = "client")]
    Client(Ident),
    #[parse(keyword = "coerce")]
    Coerce(Ident),
    #[parse(keyword = "const")]
    Const(Ident),
    #[parse(keyword = "editoronly")]
    EditorOnly(Ident),
    #[parse(keyword = "exec")]
    Exec(Ident),
    #[parse(keyword = "expensive")]
    Expensive(Ident),
    #[parse(keyword = "final")]
    Final(Ident),
    #[parse(keyword = "iterator")]
    Iterator(Ident),
    #[parse(keyword = "latent")]
    Latent(Ident),
    #[parse(keyword = "multicast")]
    Multicast(Ident),
    #[parse(keyword = "native")]
    Native(Ident, Option<ParenInt>),
    #[parse(keyword = "noexport")]
    NoExport(Ident),
    #[parse(keyword = "noexportheader")]
    NoExportHeader(Ident),
    #[parse(keyword = "noownerreplication")]
    NoOwnerReplication(Ident),
    #[parse(keyword = "public")]
    Public(Ident),
    #[parse(keyword = "private")]
    Private(Ident),
    #[parse(keyword = "protected")]
    Protected(Ident),
    #[parse(keyword = "reliable")]
    Reliable(Ident),
    #[parse(keyword = "server")]
    Server(Ident),
    #[parse(keyword = "simulated")]
    Simulated(Ident),
    #[parse(keyword = "singular")]
    Singular(Ident),
    #[parse(keyword = "static")]
    Static(Ident),
    #[parse(keyword = "unreliable")]
    Unreliable(Ident),
    #[parse(keyword = "virtual")]
    Virtual(Ident),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ParenInt {
    pub left: LeftParen,
    pub number: IntLit,
    pub right: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "kind_error")]
pub enum FunctionKind {
    #[parse(keyword = "function")]
    Function(Ident),
    #[parse(keyword = "event")]
    Event(Ident),
    #[parse(keyword = "delegate")]
    Delegate(Ident),
    #[parse(keyword = "operator")]
    Operator(Ident, ParenInt),
    #[parse(keyword = "preoperator")]
    PreOperator(Ident),
    #[parse(keyword = "postoperator")]
    PostOperator(Ident),
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Params {
    pub open: LeftParen,
    pub params: Vec<Param>,
    pub close: RightParen,
}

#[derive(Debug, Clone, Spanned)]
pub struct Param {
    pub specifiers: Vec<ParamSpecifier>,
    pub ty: Type,
    pub name: Ident,
    pub array: Option<VarArray>,
    pub default: Option<ParamDefault>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "param_specifier_error")]
pub enum ParamSpecifier {
    #[parse(keyword = "coerce")]
    Coerce(Ident),
    #[parse(keyword = "const")]
    Const(Ident),
    #[parse(keyword = "init")]
    Init(Ident),
    #[parse(keyword = "optional")]
    Optional(Ident),
    #[parse(keyword = "out")]
    Out(Ident),
    #[parse(keyword = "skip")]
    Skip(Ident),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ParamDefault {
    pub equals: Assign,
    pub value: Expr,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "body_error")]
pub enum Body {
    Stub(Semi),
    Impl(Block),
}

impl ItemFunction {
    fn parse_name(parser: &mut Parser<'_, impl TokenStream>) -> Result<Ident, ParseError> {
        parser.parse_with_error::<Ident>(|parser, span| {
            Diagnostic::error("function name expected")
                .with_label(labels::invalid_identifier(span, &parser.sources))
                .with_note(notes::IDENTIFIER_CHARS)
        })
    }
}

impl Parse for ItemFunction {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let pre_specifiers = parser.parse_greedy_list()?;
        let function = parser.parse()?;
        let post_specifiers = parser.parse_greedy_list()?;

        let (return_ty, name) = match &function {
            FunctionKind::Function(_) | FunctionKind::Event(_) | FunctionKind::Delegate(_) => {
                // NOTE: We need to do a little dance to parse return types (though it's still 100%
                // possible to do predictively, which is very nice.) This would've been easier if
                // return types weren't optional, but alas.
                let name_or_type = Self::parse_name(parser)?;
                if parser.peek_token().kind == TokenKind::LeftParen {
                    (
                        None,
                        ItemName {
                            span: name_or_type.span(),
                        },
                    )
                } else {
                    let path = Path::continue_parsing(parser, name_or_type)?;
                    let generic = parser.parse()?;
                    (
                        Some(Type {
                            specifiers: vec![],
                            path,
                            generic,
                            cpptemplate: None,
                        }),
                        ItemName {
                            span: Self::parse_name(parser)?.span(),
                        },
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
                let operator = parser.next_token();
                if !operator.kind.is_overloadable_operator() {
                    parser.emit_diagnostic(
                        Diagnostic::error(format!(
                            "`{}` is not an overloadable operator",
                            parser.sources.source(&operator)
                        ))
                        .with_label(Label::primary(&operator, "operator expected here"))
                        .with_note(indoc!(
                            r#"
                                note: overloadable operators include:
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
                    operator.span().join(&assign.span())
                } else {
                    operator.span()
                };
                (
                    Some(Type {
                        specifiers: vec![],
                        // Ugly hack to work around the fact that `int <` is a valid operator
                        // overload. FML.
                        path: return_ty,
                        generic: None,
                        cpptemplate: None,
                    }),
                    ItemName { span },
                )
            }
        };

        let params = parser.parse()?;
        let kconst = parser.parse()?;
        let body = parser.parse()?;
        Ok(Self {
            pre_specifiers,
            kind: function,
            post_specifiers,
            return_ty,
            name,
            params,
            kconst,
            body,
        })
    }
}

impl PredictiveParse for ItemFunction {
    #[allow(deprecated)]
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool {
        // Kind of sub-optimal that we have to check here each and every single identifier.
        FunctionKind::started_by(token, sources) || FunctionSpecifier::started_by(token, sources)
    }
}

impl Parse for Params {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftParen = parser.parse()?;
        let (params, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
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
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        Ok(Self {
            specifiers: parser.parse_greedy_list()?,
            ty: parser.parse()?,
            name: parser.parse()?,
            array: parser.parse()?,
            default: parser.parse()?,
        })
    }
}

impl PredictiveParse for Param {
    #[allow(deprecated)]
    fn started_by(token: &AnyToken, sources: &LexedSources<'_>) -> bool {
        Ident::started_by(token, sources)
    }
}

fn function_specifier_error(
    parser: &Parser<'_, impl TokenStream>,
    token: &AnyToken,
) -> Diagnostic<Token> {
    Diagnostic::error(format!(
        "unknown function specifier `{}`",
        parser.sources.source(token)
    ))
    .with_label(Label::primary(token, "this specifier is not recognized"))
    .with_note("note: notable function specifiers include `static` and `final`")
}

fn param_specifier_error(
    parser: &Parser<'_, impl TokenStream>,
    token: &AnyToken,
) -> Diagnostic<Token> {
    Diagnostic::error(format!(
        "unknown parameter specifier `{}`",
        parser.sources.source(token)
    ))
    .with_label(Label::primary(token, "this specifier is not recognized"))
    .with_note("note: notable parameter specifiers include `optional` and `out`")
}

fn kind_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("`function`, `event`, `preoperator`, or `operator` expected")
        .with_label(Label::primary(
            token,
            "this token does not start a function",
        ))
        .with_note("help: maybe you typo'd a specifier?")
}

fn body_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("function body `{ .. }` expected")
        .with_label(Label::primary(token, "`{` expected here"))
        .with_note(
            "note: functions can also be stubbed out using `;`, but it's probably not what you want"
        )
}
