use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    cst::{Expr, Precedence},
    lexis::token::{Colon, LeftParen, RightParen, Semi, Token},
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{Block, Stmt};

keyword! {
    KIf = "if",
    KElse = "else",
    KWhile = "while",
    KDo = "do",
    KUntil = "until",
    KFor = "for",
    KForEach = "foreach",
    KSwitch = "switch",
    KCase = "case",

    KReturn = "return",
    KBreak = "break",
    KContinue = "continue",
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct ParenExpr {
    pub open: LeftParen,
    pub expr: Expr,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtIf {
    pub kif: KIf,
    pub cond: ParenExpr,
    pub true_branch: Box<Stmt>,
    pub false_branch: Option<Else>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Else {
    pub kelse: KElse,
    pub then: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtWhile {
    pub kwhile: KWhile,
    pub cond: ParenExpr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtDo {
    pub kdo: KDo,
    pub block: Block,
    pub until: KUntil,
    pub cond: ParenExpr,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
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

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct StmtForEach {
    pub foreach: KForEach,
    pub iterator: Expr,
    pub stmt: Box<Stmt>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtSwitch {
    pub switch: KSwitch,
    pub value: ParenExpr,
    pub block: Block,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtCase {
    pub case: KCase,
    pub cond: Expr,
    pub colon: Colon,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtReturn {
    pub kreturn: KReturn,
    pub value: ReturnValue,
}

#[derive(Debug, Clone, Parse, Spanned)]
#[parse(error = "_return_value_error")]
pub enum ReturnValue {
    Nothing(Semi),
    // TODO: Overwrite error message here to instead be _return_value_error.
    #[parse(fallback)]
    Something(Expr, Semi),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtBreak {
    pub kreturn: KBreak,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct StmtContinue {
    pub kreturn: KContinue,
    pub semi: Semi,
}

impl Parse for StmtForEach {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let foreach = parser.parse()?;
        let iterator = Expr::precedence_parse(parser, Precedence::BELOW_CALL, false)?;
        let stmt = Box::new(parser.parse()?);
        Ok(Self {
            foreach,
            iterator,
            stmt,
        })
    }
}

fn _return_value_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "return value or `;` expected")
        .with_label(Label::primary(token.span, "return value expected here"))
}
