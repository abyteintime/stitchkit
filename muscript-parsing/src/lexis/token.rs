use std::fmt;

use muscript_foundation::source::{Span, Spanned};

use crate::{Parse, ParseError, ParseStream, Parser, PredictiveParse};

#[macro_export]
macro_rules! debug_token {
    ($T:ty) => {
        impl ::std::fmt::Debug for $T {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{} @ {:?}", stringify!($T), self.span)
            }
        }
    };
}

macro_rules! define_tokens {
    ($($name:tt = $pretty_name:tt),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum TokenKind {
            $($name),*
        }

        $(
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct $name {
                pub span: Span,
            }

            $crate::debug_token!($name);

            impl From<$name> for Token {
                #[track_caller]
                fn from(specific: $name) -> Self {
                    Self {
                        kind: TokenKind::$name,
                        span: specific.span,
                    }
                }
            }

            impl Spanned for $name {
                fn span(&self) -> Span {
                    self.span
                }
            }

            impl SingleToken for $name {
                const NAME: &'static str = $pretty_name;
                const KIND: TokenKind = TokenKind::$name;

                fn default_from_span(span: Span) -> Self {
                    Self { span }
                }

                fn try_from_token(token: Token, _: &str) -> Result<Self, TokenKindMismatch<Self>> {
                    if token.kind == TokenKind::$name {
                        Ok(Self { span: token.span })
                    } else {
                        Err(TokenKindMismatch(Self { span: token.span }))
                    }
                }

                fn matches(token: &Token, _: &str) -> bool {
                    token.kind == TokenKind::$name
                }
            }

            impl Parse for $name {
                fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
                    parser.expect_token()
                }
            }

            impl PredictiveParse for $name {
                fn started_by(token: &Token, _: &str) -> bool {
                    token.kind == TokenKind::$name
                }
            }
        )*
    };
}

define_tokens! {
    Comment = "comment",

    Ident = "identifier",

    Int    = "int literal",
    IntHex = "hexadecimal int literal",
    Float  = "float literal",
    String = "string literal",
    Name   = "name literal",

    Add              = "`+`",
    Sub              = "`-`",
    Mul              = "`*`",
    Div              = "`/`",
    Rem              = "`%`",
    Pow              = "`**`",
    Dollar           = "`$`",
    At               = "`@`",
    ShiftLeft        = "`<<`",
    ShiftRight       = "`>>`",
    TripleShiftRight = "`>>>`",
    BitNot           = "`~`",
    BitAnd           = "`&`",
    BitOr            = "`|`",
    BitXor           = "`^`",
    Not              = "`!`",
    Equal            = "`==`",
    NotEqual         = "`!=`",
    ApproxEqual      = "`~=`",
    Less             = "`<`",
    Greater          = "`>`",
    LessEqual        = "`<=`",
    GreaterEqual     = "`>=`",
    And              = "`&&`",
    Or               = "`||`",
    Xor              = "`^^`",
    Inc              = "`++`",
    Dec              = "`--`",

    Assign           = "`=`",
    Question         = "`?`",
    Colon            = "`:`",
    Dot              = "`.`",

    LeftParen    = "`(`",
    RightParen   = "`)`",
    LeftBracket  = "`[`",
    RightBracket = "`]`",
    LeftBrace    = "`{`",
    RightBrace   = "`}`",
    Comma        = "`,`",
    Semi         = "`;`",
    Hash         = "`#`",
    Accent       = "```", // kinda hard to decipher?
    Backslash    = "`\\`",

    // This kind is used for `isdefined and `notdefined, which should produce a valid token so that
    // they can be seen by `if, but should not be usable otherwise.
    Generated = "macro output",
    EndOfFile = "end of file",
}

impl TokenKind {
    pub fn is_overloadable_operator(&self) -> bool {
        (*self >= TokenKind::Add && *self <= TokenKind::Dec) || *self == TokenKind::Ident
    }

    pub fn can_be_compound_assignment(&self) -> bool {
        *self >= TokenKind::Add && *self <= TokenKind::BitXor
    }

    pub fn closed_by(&self) -> Option<TokenKind> {
        match self {
            TokenKind::LeftParen => Some(TokenKind::RightParen),
            TokenKind::LeftBracket => Some(TokenKind::RightBracket),
            TokenKind::LeftBrace => Some(TokenKind::RightBrace),
            _ => None,
        }
    }

    pub fn closes(&self) -> Option<TokenKind> {
        match self {
            TokenKind::RightParen => Some(TokenKind::LeftParen),
            TokenKind::RightBracket => Some(TokenKind::LeftBracket),
            TokenKind::RightBrace => Some(TokenKind::LeftBrace),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.kind, self.span)
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}

impl Parse for Token {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        parser.next_token()
    }
}

impl PredictiveParse for Token {
    fn started_by(_: &Token, _: &str) -> bool {
        true
    }
}

pub trait SingleToken: Spanned + Into<Token> + Parse + PredictiveParse {
    const NAME: &'static str;
    const KIND: TokenKind;

    fn default_from_span(span: Span) -> Self;

    fn try_from_token(token: Token, input: &str) -> Result<Self, TokenKindMismatch<Self>>;

    fn matches(token: &Token, input: &str) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenKindMismatch<T>(pub T);

#[macro_use]
mod keyword;

pub use keyword::*;
