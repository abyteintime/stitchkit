use unicase::UniCase;

use crate::lexis::token::SingleToken;

pub trait Keyword: SingleToken {
    const KEYWORD: &'static str;

    fn matches(ident: &str) -> bool {
        UniCase::new(ident) == UniCase::ascii(Self::KEYWORD)
    }
}

macro_rules! keyword {
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
                if <$T as $crate::parsing::Keyword>::matches(input) {
                    Ok(Self { span: ident.span })
                } else {
                    Err($crate::lexis::token::TokenKindMismatch(Self {
                        span: ident.span,
                    }))
                }
            }

            fn matches(_: &$crate::lexis::token::Token, input: &str) -> bool {
                <$T as $crate::parsing::Keyword>::matches(input)
            }
        }

        impl $crate::parsing::Keyword for $T {
            const KEYWORD: &'static str = $keyword;
        }

        impl ::muscript_foundation::source::Spanned for $T {
            fn span(&self) -> ::muscript_foundation::source::Span {
                self.span
            }
        }

        impl $crate::parsing::Parse for $T {
            fn parse(
                parser: &mut $crate::parsing::Parser<'_, impl $crate::lexis::TokenStream>,
            ) -> Result<Self, $crate::parsing::ParseError> {
                parser.expect_token()
            }
        }

        impl $crate::parsing::PredictiveParse for $T {
            fn starts_with(token: &$crate::lexis::token::Token, input: &str) -> bool {
                ::unicase::UniCase::new(token.span.get_input(input))
                    == ::unicase::UniCase::ascii($keyword)
            }
        }
    };
}
