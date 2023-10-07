use muscript_foundation::{
    errors::SourceRange,
    source_arena::SourceId,
    span::{Span, Spanned},
};
use std::{fmt, ops::Range};

use crate::token_stream::Channel;

pub type SourceLocation = usize;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub source_range: Range<usize>,
}

impl SourceRange for Token {
    fn source_range(&self) -> Range<usize> {
        self.source_range.clone()
    }
}

pub type TokenId = SourceId<Token>;
pub type TokenSpan = Span<Token>;

/// Passes all the token kinds as a sequence of `Token = "name",` into the provided macro.
#[macro_export]
macro_rules! expand_tokens {
    ($x:path) => {
        $x! {
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
    };
}

macro_rules! token_kind_enum {
    ($($name:tt = $pretty_name:tt),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum TokenKind {
            $($name),*
        }
    }
}

expand_tokens!(token_kind_enum);

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
        TokenSpan::Spanning {
            start: self.id,
            end: self.id,
        }
    }
}
