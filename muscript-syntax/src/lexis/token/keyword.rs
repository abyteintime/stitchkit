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
            pub span: ::muscript_foundation::source::Span,
        }

        $crate::debug_token!($T);

        impl ::std::convert::From<$T> for $crate::lexis::token::Token {
            fn from(keyword: $T) -> Self {
                Self {
                    kind: $crate::lexis::token::TokenKind::Ident,
                    span: keyword.span,
                }
            }
        }

        impl $crate::lexis::token::SingleToken for $T {
            const NAME: &'static str = concat!("`", $keyword, "`");
            const KIND: $crate::lexis::token::TokenKind = $crate::lexis::token::TokenKind::Ident;

            fn default_from_span(span: ::muscript_foundation::source::Span) -> Self {
                Self { span }
            }

            fn try_from_token(
                token: $crate::lexis::token::Token,
                input: &str,
            ) -> Result<Self, $crate::lexis::token::TokenKindMismatch<Self>> {
                let ident = $crate::lexis::token::Ident::try_from_token(token, input).map_err(
                    |$crate::lexis::token::TokenKindMismatch(ident)| {
                        $crate::lexis::token::TokenKindMismatch(Self { span: ident.span })
                    },
                )?;
                if <$T as $crate::lexis::token::Keyword>::matches(input) {
                    Ok(Self { span: ident.span })
                } else {
                    Err($crate::lexis::token::TokenKindMismatch(Self {
                        span: ident.span,
                    }))
                }
            }

            fn matches(_: &$crate::lexis::token::Token, input: &str) -> bool {
                <$T as $crate::lexis::token::Keyword>::matches(input)
            }
        }

        impl $crate::lexis::token::Keyword for $T {
            const KEYWORD: &'static str = $keyword;
        }

        impl ::muscript_foundation::source::Spanned for $T {
            fn span(&self) -> ::muscript_foundation::source::Span {
                self.span
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
            fn started_by(token: &$crate::lexis::token::Token, input: &str) -> bool {
                token.span.get_input(input).eq_ignore_ascii_case($keyword)
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
        if let Ok(next) = self.peek_token() {
            next.kind == TokenKind::Ident
                && next
                    .span
                    .get_input(self.input)
                    .eq_ignore_ascii_case(keyword)
        } else {
            false
        }
    }
}
