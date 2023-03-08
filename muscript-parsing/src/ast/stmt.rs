mod control_flow;
mod local;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{LeftBrace, RightBrace, Semi, Token},
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

pub use control_flow::*;
pub use local::*;

use super::Expr;

#[derive(Debug, Clone, Parse)]
#[parse(error = "_stmt_error")]
pub enum Stmt {
    Empty(Semi),
    Block(Block),

    Local(StmtLocal),
    If(StmtIf),
    While(StmtWhile),
    Do(StmtDo),
    For(StmtFor),

    Return(StmtReturn),
    Break(StmtBreak),
    Continue(StmtContinue),

    // TODO: Overwrite the error message here to instead be _stmt_error.
    #[parse(fallback)]
    Expr(StmtExpr),
}

#[derive(Debug, Clone, Parse)]
pub struct StmtExpr {
    pub expr: Expr,
    pub semi: Semi,
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Block {
    pub open: LeftBrace,
    pub stmts: Vec<Stmt>,
    pub close: RightBrace,
}

impl Parse for Block {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "block `{ .. }` expected")
                .with_label(Label::primary(span, "`{` expected here"))
        })?;
        let (stmts, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `}` to close block")
                        .with_label(Label::primary(open.span, "this is where the block begins")),
                );
            }
            error.parse
        })?;
        Ok(Self { open, stmts, close })
    }
}

fn _stmt_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "statement expected")
        .with_label(Label::primary(
            token.span,
            "this token does not start a statement",
        ))
        .with_note("note: notable statement types include expressions, `local`, `if`, `while`, `for`, etc.")
}
