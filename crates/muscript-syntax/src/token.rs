#[macro_use]
mod keyword;
mod parsing;

use muscript_foundation::span::Spanned;
use muscript_lexer::{
    expand_tokens,
    sources::LexedSources,
    token::{Token, TokenId, TokenKind, TokenSpan},
    token_stream::{Channel, TokenStream},
};

use crate::{Parse, ParseError, Parser, PredictiveParse};

pub use muscript_lexer::token::AnyToken;

pub use keyword::*;

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

impl Parse for AnyToken {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
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

macro_rules! strong_token_types {
    ($($name:tt = $pretty_name:tt),* $(,)?) => {
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
                    TokenSpan::Spanning {
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
                fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
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

expand_tokens!(strong_token_types);
