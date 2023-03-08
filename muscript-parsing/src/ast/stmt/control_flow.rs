use muscript_foundation::errors::{Diagnostic, Label};
use muscript_parsing_derive::{Parse, PredictiveParse};

use crate::{
    ast::Expr,
    lexis::token::{LeftParen, RightParen, Semi, Token},
    ParseStream, Parser,
};

use super::{Block, Stmt};

keyword! {
    KIf = "if",
    KElse = "else",
    KWhile = "while",
    KDo = "do",
    KUntil = "until",
    KFor = "for",
    KSwitch = "switch",
    KCase = "case",
    KDefault = "default",

    KReturn = "return",
    KBreak = "break",
    KContinue = "continue",
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Cond {
    pub open: LeftParen,
    pub cond: Expr,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtIf {
    pub kif: KIf,
    pub cond: Cond,
    pub true_branch: Box<Stmt>,
    pub false_branch: Option<Else>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Else {
    pub kelse: KElse,
    pub then: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtWhile {
    pub kwhile: KWhile,
    pub cond: Cond,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtDo {
    pub kdo: KDo,
    pub block: Block,
    pub until: KUntil,
    pub cond: Cond,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtFor {
    pub kfor: KFor,

    pub open: LeftParen,
    pub init: Expr,
    pub semi1: Semi,
    pub cond: Expr,
    pub semi2: Semi,
    pub update: Expr,
    pub close: RightParen,

    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtReturn {
    pub kreturn: KReturn,
    pub value: ReturnValue,
}

#[derive(Debug, Clone, Parse)]
#[parse(error = "_return_value_error")]
pub enum ReturnValue {
    Nothing(Semi),
    // TODO: Overwrite error message here to instead be _return_value_error.
    #[parse(fallback)]
    Something(Expr, Semi),
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtBreak {
    pub kreturn: KBreak,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct StmtContinue {
    pub kreturn: KContinue,
    pub semi: Semi,
}

fn _return_value_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "return value or `;` expected")
        .with_label(Label::primary(token.span, "return value expected here"))
}
