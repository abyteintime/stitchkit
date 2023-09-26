mod control_flow;
mod local;

use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    lexis::token::{AnyToken, LeftBrace, RightBrace, Semi, Token},
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

pub use control_flow::*;
pub use local::*;

use super::{Expr, Precedence};

#[derive(Debug, Clone, Parse, Spanned)]
#[parse(error = "_stmt_error")]
pub enum Stmt {
    Empty(Semi),
    Block(Block),

    Local(StmtLocal),
    If(StmtIf),
    While(StmtWhile),
    Do(StmtDo),
    For(StmtFor),
    ForEach(StmtForEach),
    Switch(StmtSwitch),

    Case(StmtCase),
    Return(StmtReturn),
    Break(StmtBreak),
    Continue(StmtContinue),

    // TODO: Overwrite the error message here to instead be _stmt_error.
    #[parse(fallback)]
    Expr(StmtExpr),
}

#[derive(Debug, Clone, Spanned)]
pub struct StmtExpr {
    pub expr: Expr,
    pub semi: Option<Semi>,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Block {
    pub open: LeftBrace,
    pub stmts: Vec<Stmt>,
    pub close: RightBrace,
}

impl Parse for StmtExpr {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let expr = Expr::precedence_parse(parser, Precedence::EXPR, true)?;
        if let Expr::Label { .. } = &expr {
            Ok(Self { expr, semi: None })
        } else {
            Ok(Self {
                expr,
                semi: parser.parse()?,
            })
        }
    }
}

impl Parse for Block {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse_with_error(|_, span| {
            Diagnostic::error("block `{ .. }` expected")
                .with_label(Label::primary(&span, "`{` expected here"))
        })?;
        let (stmts, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error("missing `}` to close block")
                        .with_label(Label::primary(&open, "this is where the block begins")),
                );
            }
            error.parse
        })?;
        Ok(Self { open, stmts, close })
    }
}

fn _stmt_error(_: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("statement expected")
        .with_label(Label::primary(
            token,
            "this token does not start a statement",
        ))
        .with_note("note: notable statement types include expressions, `local`, `if`, `while`, `for`, etc.")
}
