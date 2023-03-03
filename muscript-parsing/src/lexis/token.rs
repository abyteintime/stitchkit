use std::fmt;

use muscript_foundation::source::{Span, Spanned};

use crate::parsing::{Parse, ParseError, Parser, PredictiveParse};

use super::TokenStream;

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
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
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

    None  = "`none`",
    True  = "`true`",
    False = "`false`",

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
    Colon            = "`:`",
    Question         = "`?`",
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

    LeftParen    = "`(`",
    RightParen   = "`)`",
    LeftBracket  = "`[`",
    RightBracket = "`]`",
    LeftBrace    = "`{`",
    RightBrace   = "`}`",
    Dot          = "`.`",
    Comma        = "``",
    Semi    = "`;`",
    Hash         = "`#`",
    Accent       = "```", // kinda hard to decipher?
    Backslash    = "`\\`",

    EndOfFile = "end of file",
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

pub trait SingleToken: Spanned + Into<Token> + Parse + PredictiveParse {
    const NAME: &'static str;

    fn default_from_span(span: Span) -> Self;

    fn try_from_token(token: Token, input: &str) -> Result<Self, TokenKindMismatch<Self>>;

    fn matches(token: &Token, input: &str) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenKindMismatch<T>(pub T);
