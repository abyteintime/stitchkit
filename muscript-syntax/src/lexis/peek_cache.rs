use muscript_foundation::source::Span;

use super::{token::Token, Channel, EofReached, LexError, LexicalContext, TokenStream};

/// Adds a cache for all peek (but not next) operations.
pub struct PeekCaching<T> {
    inner: T,
    cache: Option<Cache>,
}

struct Cache {
    token: Token,
    context: LexicalContext,
    channel: Channel,
}

impl<T> PeekCaching<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, cache: None }
    }
}

impl<T> TokenStream for PeekCaching<T>
where
    T: TokenStream,
{
    fn next_any(&mut self, context: LexicalContext) -> Result<Token, LexError> {
        self.cache = None;
        self.inner.next_any(context)
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached> {
        self.inner.text_blob(is_end)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        self.inner.braced_string(left_brace_span)
    }

    fn peek_from(&mut self, context: LexicalContext, channel: Channel) -> Result<Token, LexError> {
        if let Some(cache) = &self.cache {
            if cache.context == context && cache.channel == channel {
                return Ok(cache.token.clone());
            }
        }

        let token = self.inner.peek_from(context, channel)?;
        self.cache = Some(Cache {
            token: token.clone(),
            context,
            channel,
        });
        Ok(token)
    }
}
