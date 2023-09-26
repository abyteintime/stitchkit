use crate::{lexis::token::SingleToken, ParseStream, Parser};

use super::TokenKind;

pub trait Keyword: SingleToken {
    const KEYWORD: &'static str;

    fn matches(ident: &str) -> bool {
        ident.eq_ignore_ascii_case(Self::KEYWORD)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __keyword_impl {
    ($T:tt = $keyword:tt) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $T {
            pub id: $crate::lexis::token::TokenId,
        }

        $crate::debug_token!($T);

        impl ::std::convert::From<$T> for $crate::lexis::token::AnyToken {
            fn from(keyword: $T) -> Self {
                Self {
                    kind: $crate::lexis::token::TokenKind::Ident,
                    id: keyword.id,
                }
            }
        }

        impl $crate::lexis::token::SingleToken for $T {
            const NAME: &'static str = concat!("`", $keyword, "`");
            const KIND: $crate::lexis::token::TokenKind = $crate::lexis::token::TokenKind::Ident;

            fn id(&self) -> $crate::lexis::token::TokenId {
                self.id
            }

            fn default_from_id(id: $crate::lexis::token::TokenId) -> Self {
                Self { id }
            }

            fn try_from_token(
                token: $crate::lexis::token::AnyToken,
                sources: &$crate::sources::LexedSources<'_>,
            ) -> Result<Self, $crate::lexis::token::TokenKindMismatch> {
                let ident = $crate::lexis::token::Ident::try_from_token(token, sources)?;
                if <$T as $crate::lexis::token::Keyword>::matches(sources.source(&token)) {
                    Ok(Self { id: ident.id })
                } else {
                    Err($crate::lexis::token::TokenKindMismatch { token_id: ident.id })
                }
            }

            fn matches(
                token: &$crate::lexis::token::AnyToken,
                sources: &$crate::sources::LexedSources<'_>,
            ) -> bool {
                <$T as $crate::lexis::token::Keyword>::matches(sources.source(token))
            }
        }

        impl $crate::lexis::token::Keyword for $T {
            const KEYWORD: &'static str = $keyword;
        }

        impl ::muscript_foundation::span::Spanned<$crate::lexis::token::Token> for $T {
            fn span(&self) -> $crate::lexis::token::TokenSpan {
                $crate::lexis::token::TokenSpan::single(self.id)
            }
        }

        impl $crate::parsing::Parse for $T {
            fn parse(
                parser: &mut $crate::parsing::Parser<'_, impl $crate::ParseStream>,
            ) -> Result<Self, $crate::parsing::ParseError> {
                parser.expect_token()
            }
        }

        impl $crate::parsing::PredictiveParse for $T {
            fn started_by(
                token: &$crate::lexis::token::AnyToken,
                sources: &$crate::sources::LexedSources<'_>,
            ) -> bool {
                sources.source(token).eq_ignore_ascii_case($keyword)
            }
        }
    };
}

macro_rules! keyword {
    ($($T:tt = $keyword:tt),* $(,)?) => {
        $(
            $crate::__keyword_impl!($T = $keyword);
        )*
    }
}

// Fast keyword parsing that does not slow down compile times nearly as much as `keyword!` does.
// This is expected to replace `keyword!` entirely.
impl<'a, T> Parser<'a, T>
where
    T: ParseStream,
{
    pub fn next_matches_keyword(&mut self, keyword: &str) -> bool {
        let next = self.peek_token();
        next.kind == TokenKind::Ident && self.sources.source(&next).eq_ignore_ascii_case(keyword)
    }
}
