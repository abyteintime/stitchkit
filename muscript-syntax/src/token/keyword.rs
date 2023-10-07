use muscript_lexer::token_stream::TokenStream;

use crate::{token::SingleToken, Parser};

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
            pub id: muscript_lexer::token::TokenId,
        }

        $crate::debug_token!($T);

        impl ::std::convert::From<$T> for muscript_lexer::token::AnyToken {
            fn from(keyword: $T) -> Self {
                Self {
                    kind: muscript_lexer::token::TokenKind::Ident,
                    id: keyword.id,
                }
            }
        }

        impl $crate::token::SingleToken for $T {
            const NAME: &'static str = concat!("`", $keyword, "`");
            const KIND: muscript_lexer::token::TokenKind = muscript_lexer::token::TokenKind::Ident;

            fn id(&self) -> muscript_lexer::token::TokenId {
                self.id
            }

            fn default_from_id(id: muscript_lexer::token::TokenId) -> Self {
                Self { id }
            }

            fn try_from_token(
                token: muscript_lexer::token::AnyToken,
                sources: &muscript_lexer::sources::LexedSources<'_>,
            ) -> Result<Self, $crate::token::TokenKindMismatch> {
                let ident = $crate::token::Ident::try_from_token(token, sources)?;
                if <$T as $crate::token::Keyword>::matches(sources.source(&token)) {
                    Ok(Self { id: ident.id })
                } else {
                    Err($crate::token::TokenKindMismatch { token_id: ident.id })
                }
            }

            fn matches(
                token: &muscript_lexer::token::AnyToken,
                sources: &muscript_lexer::sources::LexedSources<'_>,
            ) -> bool {
                <$T as $crate::token::Keyword>::matches(sources.source(token))
            }
        }

        impl $crate::token::Keyword for $T {
            const KEYWORD: &'static str = $keyword;
        }

        impl ::muscript_foundation::span::Spanned<::muscript_lexer::token::Token> for $T {
            fn span(&self) -> muscript_lexer::token::TokenSpan {
                muscript_lexer::token::TokenSpan::single(self.id)
            }
        }

        impl $crate::parsing::Parse for $T {
            fn parse(
                parser: &mut $crate::parsing::Parser<
                    '_,
                    impl ::muscript_lexer::token_stream::TokenStream,
                >,
            ) -> Result<Self, $crate::parsing::ParseError> {
                parser.expect_token()
            }
        }

        impl $crate::parsing::PredictiveParse for $T {
            fn started_by(
                token: &muscript_lexer::token::AnyToken,
                sources: &muscript_lexer::sources::LexedSources<'_>,
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
    T: TokenStream,
{
    pub fn next_matches_keyword(&mut self, keyword: &str) -> bool {
        let next = self.peek_token();
        next.kind == TokenKind::Ident && self.sources.source(&next).eq_ignore_ascii_case(keyword)
    }
}
