mod lit;

use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    lexis::token::{self, Float, Ident, Int, IntHex, Keyword, Name, Token, TokenKind},
    Parse, ParseError, ParseStream, Parser,
};

pub use lit::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Ident(Ident),
    Unary(Unary),
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub expr: Box<Expr>,
}

// Expression parsing is not implemented using regular recursive descent because of two reasons:
//  - precedence
//  - performance
// To maintain precedence we need use Pratt parsing (precedence climbing). It would be very
// annoying, imperformant, and hard to maintain if we did the usual trick of defining one rule
// for each precedence level.
impl Expr {
    fn parse_prefix(
        parser: &mut Parser<'_, impl ParseStream>,
        token: Token,
    ) -> Result<Expr, ParseError> {
        Ok(match token.kind {
            TokenKind::Ident => Expr::ident(parser, token)?,
            TokenKind::Int => Expr::Lit(Lit::Int(IntLit::Dec(Int { span: token.span }))),
            TokenKind::IntHex => Expr::Lit(Lit::Int(IntLit::Hex(IntHex { span: token.span }))),
            TokenKind::Float => Expr::Lit(Lit::Float(Float { span: token.span })),
            TokenKind::String => Expr::Lit(Lit::String(token::String { span: token.span })),
            TokenKind::Name => Expr::Lit(Lit::Name(Name { span: token.span })),

            TokenKind::Sub | TokenKind::BitNot => Expr::unary(parser, token)?,

            _ => parser.bail(
                token.span,
                // NOTE: This error message specifically avoids mentioning the concept of prefix
                // tokens, since they're not actually relevant to what's happening here.
                // What is *really* happening is that we expect any ol' expression, but the user
                // gave us something that isn't.
                Diagnostic::error(parser.file, "expression expected")
                    .with_label(Label::primary(
                        token.span,
                        "this token does not start an expression",
                    ))
                    .with_note("note: expression types include literals, variables, math, etc."),
            )?,
        })
    }

    fn unary(
        parser: &mut Parser<'_, impl ParseStream>,
        operator: Token,
    ) -> Result<Expr, ParseError> {
        Ok(Expr::Unary(Unary {
            operator,
            expr: {
                let token = parser.next_token()?;
                Box::new(Self::parse_prefix(parser, token)?)
            },
        }))
    }

    fn ident(parser: &mut Parser<'_, impl ParseStream>, ident: Token) -> Result<Expr, ParseError> {
        let s = ident.span.get_input(parser.input);
        Ok(match () {
            _ if KNone::matches(s) => Expr::Lit(Lit::None(KNone { span: ident.span })),
            _ if KTrue::matches(s) => {
                Expr::Lit(Lit::Bool(BoolLit::True(KTrue { span: ident.span })))
            }
            _ if KFalse::matches(s) => {
                Expr::Lit(Lit::Bool(BoolLit::False(KFalse { span: ident.span })))
            }
            _ => Expr::Ident(Ident { span: ident.span }),
        })
    }

    fn parse_infix(
        parser: &mut Parser<'_, impl ParseStream>,
        left: Expr,
        token: Token,
    ) -> Result<Expr, ParseError> {
        todo!("expression parsing / precedence climbing")
    }

    fn precedence_parse(
        parser: &mut Parser<'_, impl ParseStream>,
        precedence: u8,
    ) -> Result<Expr, ParseError> {
        let first = parser.next_token()?;
        Expr::parse_prefix(parser, first)
    }
}

impl Parse for Expr {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Expr::precedence_parse(parser, 0)
    }
}
