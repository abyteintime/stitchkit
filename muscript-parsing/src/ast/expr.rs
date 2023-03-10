mod lit;

use std::cmp::Ordering;

use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{Span, Spanned},
};
use unicase::UniCase;

use crate::{
    lexis::token::{
        Assign, Colon, Dot, FloatLit, Ident, IntLit, Keyword, LeftBracket, LeftParen, NameLit,
        Question, RightBracket, RightParen, StringLit, Token, TokenKind,
    },
    list::SeparatedListDiagnostics,
    Parse, ParseError, ParseStream, Parser,
};

pub use lit::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Ident(Ident),
    Object {
        class: Ident,
        name: NameLit,
    },

    Prefix {
        operator: Token,
        right: Box<Expr>,
    },
    Postfix {
        left: Box<Expr>,
        operator: Token,
    },
    Binary {
        left: Box<Expr>,
        operator: InfixOperator,
        right: Box<Expr>,
    },
    Paren {
        open: LeftParen,
        inner: Box<Expr>,
        close: RightParen,
    },

    Assign {
        lvalue: Box<Expr>,
        assign: Assign,
        rvalue: Box<Expr>,
    },
    Dot {
        left: Box<Expr>,
        dot: Dot,
        field: Ident,
    },
    Index {
        left: Box<Expr>,
        open: LeftBracket,
        index: Box<Expr>,
        close: RightBracket,
    },
    Call {
        function: Box<Expr>,
        open: LeftParen,
        args: Vec<Expr>,
        close: RightParen,
    },
    Ternary {
        cond: Box<Expr>,
        question: Question,
        true_result: Box<Expr>,
        colon: Colon,
        false_result: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfixOperator {
    pub token: Token,
    pub assign: Option<Assign>,
}

impl Spanned for InfixOperator {
    fn span(&self) -> Span {
        self.assign
            .map(|assign| assign.span.join(&self.token.span))
            .unwrap_or(self.token.span)
    }
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
        use crate::lexis::token::SingleToken;

        Ok(match token.kind {
            TokenKind::Ident => Expr::ident(parser, token)?,
            TokenKind::IntLit => Expr::Lit(Lit::Int(IntLit { span: token.span })),
            TokenKind::FloatLit => Expr::Lit(Lit::Float(FloatLit { span: token.span })),
            TokenKind::StringLit => Expr::Lit(Lit::String(StringLit { span: token.span })),
            TokenKind::NameLit => Expr::Lit(Lit::Name(NameLit { span: token.span })),

            TokenKind::Sub
            | TokenKind::Not
            | TokenKind::BitNot
            | TokenKind::Inc
            | TokenKind::Dec => Expr::unary(parser, token)?,

            TokenKind::LeftParen => {
                let inner = Expr::precedence_parse(parser, Precedence::EXPR)?;
                let close = parser.parse_with_error(|parser, span| {
                    Diagnostic::error(parser.file, "missing `)` to close grouped expression")
                        .with_label(Label::primary(span, "`)` expected here..."))
                        .with_label(Label::secondary(token.span, "...to close this `(`"))
                })?;
                Expr::Paren {
                    open: LeftParen::default_from_span(token.span),
                    inner: Box::new(inner),
                    close,
                }
            }

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
        Ok(Expr::Prefix {
            operator,
            right: {
                let token = parser.next_token()?;
                Box::new(Self::parse_prefix(parser, token)?)
            },
        })
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
            _ => {
                let ident = Ident { span: ident.span };
                if let Some(name) = parser.parse()? {
                    Expr::Object { class: ident, name }
                } else {
                    Expr::Ident(ident)
                }
            }
        })
    }

    fn parse_infix(
        parser: &mut Parser<'_, impl ParseStream>,
        left: Expr,
        op: InfixOperator,
    ) -> Result<Expr, ParseError> {
        use crate::lexis::token::SingleToken;

        Ok(match op.token.kind {
            TokenKind::Inc | TokenKind::Dec => Expr::Postfix {
                left: Box::new(left),
                operator: op.token,
            },
            _ if op.token.kind.is_overloadable_operator() => {
                Expr::binary(parser, op, move |op, right| Expr::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                })?
            }

            TokenKind::Assign => Expr::binary(parser, op, move |op, right| Expr::Assign {
                lvalue: Box::new(left),
                assign: Assign::default_from_span(op.token.span),
                rvalue: Box::new(right),
            })?,
            TokenKind::Dot => Expr::Dot {
                left: Box::new(left),
                dot: Dot::default_from_span(op.token.span),
                field: parser.parse()?,
            },
            TokenKind::LeftParen => Expr::function_call(parser, left, op.token)?,
            TokenKind::LeftBracket => Expr::Index {
                left: Box::new(left),
                open: LeftBracket::default_from_span(op.token.span),
                index: Box::new(Expr::precedence_parse(parser, Precedence::EXPR)?),
                close: parser.parse()?,
            },
            TokenKind::Question => Expr::ternary(parser, left, op.token)?,

            _ => parser.bail(
                op.token.span,
                Diagnostic::bug(parser.file, "unimplemented infix operator")
                    .with_label(Label::primary(op.span(), "this operator cannot be parsed"))
                    .with_note("note: this means an infix operator was given a precedence level, but wasn't matched by Expr::parse_infix"),
            )?,
        })
    }

    fn binary(
        parser: &mut Parser<'_, impl ParseStream>,
        operator: InfixOperator,
        build: impl FnOnce(InfixOperator, Expr) -> Expr,
    ) -> Result<Expr, ParseError> {
        let right = Expr::precedence_parse(parser, operator.token.precedence(parser.input))?;
        Ok(build(operator, right))
    }

    fn ternary(
        parser: &mut Parser<'_, impl ParseStream>,
        left: Expr,
        token: Token,
    ) -> Result<Expr, ParseError> {
        use crate::lexis::token::SingleToken;

        let precedence = token.precedence(parser.input);
        Ok(Expr::Ternary {
            cond: Box::new(left),
            question: Question::default_from_span(token.span),
            true_result: Box::new(Expr::precedence_parse(parser, precedence)?),
            colon: parser.parse()?,
            // NOTE: We want to use one less precedence here, since this should be able to match
            // ?: without a problem, so that ternaries can be chained like an if-else if-else.
            false_result: Box::new(Expr::precedence_parse(parser, Precedence::BELOW_TERNARY)?),
        })
    }

    fn function_call(
        parser: &mut Parser<'_, impl ParseStream>,
        left: Expr,
        token: Token,
    ) -> Result<Expr, ParseError> {
        use crate::lexis::token::SingleToken;

        let open = LeftParen::default_from_span(token.span);
        let (args, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `)` to close function argument list",
                    missing_right_label: "this `(` does not have a matching `)`",
                    // TODO: Is there anything to do to make this error more accurate?
                    // For example if the programmer slips their hand and types an invalid infix
                    // operator, the expression parser will halt and come here instead,
                    // which is not ideal.
                    missing_comma: "`,` or `)` expected after function argument",
                    missing_comma_open: "the argument list starts here",
                    missing_comma_token: "this was expected to continue or close the argument list",
                    missing_comma_note: "note: arguments to functions are separated by commas `,`",
                },
            )
        })?;
        Ok(Expr::Call {
            function: Box::new(left),
            open,
            args,
            close,
        })
    }

    fn next_infix_operator(
        parser: &mut Parser<'_, impl ParseStream>,
    ) -> Result<InfixOperator, ParseError> {
        use crate::lexis::token::SingleToken;

        let token = parser.next_token()?;
        let possibly_assign = parser.peek_token()?;
        if possibly_assign.kind == TokenKind::Assign && possibly_assign.span.start == token.span.end
        {
            let assign = parser.next_token()?;
            Ok(InfixOperator {
                token,
                assign: Some(Assign::default_from_span(assign.span)),
            })
        } else {
            Ok(InfixOperator {
                token,
                assign: None,
            })
        }
    }

    pub fn precedence_parse(
        parser: &mut Parser<'_, impl ParseStream>,
        precedence: Precedence,
    ) -> Result<Expr, ParseError> {
        let token = parser.next_token()?;
        let mut chain = Expr::parse_prefix(parser, token)?;

        let mut operator;
        while precedence < parser.peek_token()?.precedence(parser.input) {
            operator = Expr::next_infix_operator(parser)?;
            chain = Expr::parse_infix(parser, chain, operator)?;
        }

        Ok(chain)
    }
}

impl Precedence {
    pub const PATH: Self = Self::Some(6);
    pub const CALL: Self = Self::Some(8);
    // Needed for `foreach` statements. UnrealScript may be a little more... hardcoded in this
    // case, but I think there's a case to be made for letting your iterators be arbitrary
    // expressions.
    pub const BELOW_CALL: Self = Self::Some(9);

    // We don't know the actual precedence of postfix operators so we take a guess that
    // it's something lower than paths and higher than arithmetic.
    pub const POSTFIX: Self = Self::Some(10);

    pub const TERNARY: Self = Self::Some(48);
    pub const BELOW_TERNARY: Self = Self::Some(49);

    pub const ASSIGN: Self = Self::Some(50);

    pub const EXPR: Self = Self::Some(u8::MAX);
}

impl Token {
    fn precedence(&self, input: &str) -> Precedence {
        // Unlike vanilla UnrealScript, we hardcode our precedence numbers because not doing so
        // would make parsing insanely hard.
        match self.kind {
            _ if self.kind.is_overloadable_operator() && self.is_compound_assignment(input) => {
                Precedence::ASSIGN
            }

            // These precedence numbers are for magic operators and are only best guesses.
            TokenKind::Dot => Precedence::PATH,
            TokenKind::LeftBracket => Precedence::PATH,
            TokenKind::LeftParen => Precedence::CALL,
            TokenKind::Question => Precedence::TERNARY,
            TokenKind::Assign => Precedence::ASSIGN,

            // These precedence numbers are taken straight from Object.uc.
            TokenKind::Pow => Precedence::Some(12),
            TokenKind::Mul => Precedence::Some(16),
            TokenKind::Div => Precedence::Some(16),
            TokenKind::Rem => Precedence::Some(16),
            TokenKind::Add => Precedence::Some(20),
            TokenKind::Sub => Precedence::Some(20),
            TokenKind::ShiftLeft => Precedence::Some(22),
            TokenKind::ShiftRight => Precedence::Some(22),
            TokenKind::TripleShiftRight => Precedence::Some(22),
            TokenKind::Equal => Precedence::Some(24),
            TokenKind::Less => Precedence::Some(24),
            TokenKind::LessEqual => Precedence::Some(24),
            TokenKind::Greater => Precedence::Some(24),
            TokenKind::GreaterEqual => Precedence::Some(24),
            TokenKind::ApproxEqual => Precedence::Some(24),
            // Weird thing: != has lower precedence than ==.
            TokenKind::NotEqual => Precedence::Some(26),
            TokenKind::BitAnd => Precedence::Some(28),
            TokenKind::BitXor => Precedence::Some(28),
            TokenKind::BitOr => Precedence::Some(28),
            TokenKind::And => Precedence::Some(30),
            TokenKind::Xor => Precedence::Some(30),
            TokenKind::Or => Precedence::Some(32),
            // These two are incompatible with vanilla UnrealScript because the precedence
            // declared in Object.uc doesn't make sense. Why would `$` and `@` bind weaker than `=`?
            TokenKind::Dollar | TokenKind::At => Precedence::Some(34),

            TokenKind::Ident => match UniCase::new(self.span.get_input(input)) {
                s if s == UniCase::ascii("dot") => Precedence::Some(16),
                s if s == UniCase::ascii("cross") => Precedence::Some(16),
                s if s == UniCase::ascii("clockwisefrom") => Precedence::Some(24),
                _ => Precedence::None,
            },

            TokenKind::Inc | TokenKind::Dec => Precedence::POSTFIX,

            _ => Precedence::None,
        }
    }

    fn is_compound_assignment(&self, input: &str) -> bool {
        // This is a little bit cursed, but the level of cursedness here is nothing compared to
        // UnrealScript as a whole.
        input[self.span.end..].starts_with('=')
    }
}

/// Specialized version of [`Option<T>`] that's specialized for precedence levels.
///
/// Unlike [`Option<u8>`], it compares correctly given UnrealScript's inverted precedence hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precedence {
    None,
    Some(u8),
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let this = Option::<u8>::from(*self).map(|x| u8::MAX - x);
        let other = Option::<u8>::from(*other).map(|x| u8::MAX - x);
        this.partial_cmp(&other)
    }
}

impl Ord for Precedence {
    fn cmp(&self, other: &Self) -> Ordering {
        let this = Option::<u8>::from(*self).map(|x| u8::MAX - x);
        let other = Option::<u8>::from(*other).map(|x| u8::MAX - x);
        this.cmp(&other)
    }
}

impl From<Option<u8>> for Precedence {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(x) => Self::Some(x),
            None => Self::None,
        }
    }
}

impl From<Precedence> for Option<u8> {
    fn from(value: Precedence) -> Self {
        match value {
            Precedence::None => None,
            Precedence::Some(x) => Some(x),
        }
    }
}

impl Parse for Expr {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Expr::precedence_parse(parser, Precedence::EXPR)
    }
}
