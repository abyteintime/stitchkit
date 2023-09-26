use muscript_foundation::{
    source_arena::SourceId,
    span::{Span, Spanned},
};
use std::{fmt, ops::Range};

use crate::{sources::LexedSources, Parse, ParseError, ParseStream, Parser, PredictiveParse};

pub type SourceLocation = usize;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub source_range: Range<usize>,
}

pub type TokenId = SourceId<Token>;
pub type TokenSpan = Span<Token>;

#[macro_export]
macro_rules! debug_token {
    ($T:ty) => {
        impl ::std::fmt::Debug for $T {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}({:?})", stringify!($T), self.id)
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
                pub id: TokenId,
            }

            $crate::debug_token!($name);

            impl From<$name> for AnyToken {
                #[track_caller]
                fn from(specific: $name) -> Self {
                    Self {
                        kind: TokenKind::$name,
                        id: specific.id,
                    }
                }
            }

            impl Spanned<Token> for $name {
                fn span(&self) -> TokenSpan {
                    Span::Spanning {
                        start: self.id,
                        end: self.id,
                    }
                }
            }

            impl SingleToken for $name {
                const NAME: &'static str = $pretty_name;
                const KIND: TokenKind = TokenKind::$name;

                fn id(&self) -> TokenId {
                    self.id
                }

                fn default_from_id(id: TokenId) -> Self {
                    Self { id }
                }

                fn try_from_token(token: AnyToken, _: &LexedSources<'_>) -> Result<Self, TokenKindMismatch> {
                    if token.kind == TokenKind::$name {
                        Ok(Self { id: token.id })
                    } else {
                        Err(TokenKindMismatch { token_id: token.id })
                    }
                }

                fn matches(token: &AnyToken, _: &LexedSources<'_>) -> bool {
                    token.kind == TokenKind::$name
                }
            }

            impl Parse for $name {
                fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
                    parser.expect_token()
                }
            }

            impl PredictiveParse for $name {
                const LISTEN_TO_CHANNELS: Channel = Self::KIND.channel();

                fn started_by(token: &AnyToken, _: &LexedSources<'_>) -> bool {
                    token.kind == TokenKind::$name
                }
            }
        )*
    };
}

define_tokens! {
    Comment = "comment",

    Ident = "identifier",

    IntLit    = "int literal",
    FloatLit  = "float literal",
    StringLit = "string literal",
    NameLit   = "name literal",

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

    // Used for errors produced by the lexer.
    // These belong to the same channel as comment tokens (they are not )
    Error = "error",

    // This kind is used for `isdefined and `notdefined, which should produce a valid token so that
    // they can be seen by `if, but should not be usable otherwise.
    Generated = "macro output",
    // This kind is produced by expanding undefined macros, and is primarily used for error recovery
    // in various places.
    FailedExp = "undefined macro output",
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

    pub const fn channel(&self) -> Channel {
        match self {
            TokenKind::Comment => Channel::COMMENT,
            TokenKind::FailedExp => Channel::MACRO,
            TokenKind::Error => Channel::ERROR,
            _ => Channel::CODE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AnyToken {
    pub kind: TokenKind,
    pub id: TokenId,
}

impl fmt::Debug for AnyToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({:?})", self.kind, self.id)
    }
}

impl Spanned<Token> for AnyToken {
    fn span(&self) -> TokenSpan {
        Span::Spanning {
            start: self.id,
            end: self.id,
        }
    }
}

impl Parse for AnyToken {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        Ok(parser.next_token())
    }
}

impl PredictiveParse for AnyToken {
    fn started_by(_: &AnyToken, _: &LexedSources<'_>) -> bool {
        true
    }
}

pub trait SingleToken: Spanned<Token> + Into<AnyToken> + Parse + PredictiveParse {
    const NAME: &'static str;
    const KIND: TokenKind;

    fn id(&self) -> TokenId;

    fn default_from_id(span: TokenId) -> Self;

    fn try_from_token(
        token: AnyToken,
        sources: &LexedSources<'_>,
    ) -> Result<Self, TokenKindMismatch>;

    fn matches(token: &AnyToken, sources: &LexedSources<'_>) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenKindMismatch {
    pub token_id: TokenId,
}

#[macro_use]
mod keyword;
mod parsing;

pub use keyword::*;

use super::Channel;
