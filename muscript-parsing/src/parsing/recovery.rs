use muscript_foundation::{errors::Diagnostic, source::Span};

use crate::{
    lexis::{
        token::{SingleToken, Token, TokenKind},
        Channel, EofReached, LexError, LexicalContext, TokenStream,
    },
    ParseError, ParseStream, Parser,
};

/// Wrapper for any token stream that adds error recovery state to it.
pub struct Structured<T> {
    inner: T,
    delimiter_stack: Vec<TokenKind>,
}

impl<T> Structured<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            delimiter_stack: vec![],
        }
    }
}

impl<T> TokenStream for Structured<T>
where
    T: TokenStream,
{
    fn next_any(&mut self, context: LexicalContext) -> Result<Token, LexError> {
        let token = self.inner.next_any(context)?;

        if let Some(closing_kind) = token.kind.closed_by() {
            self.delimiter_stack.push(closing_kind);
        }
        if token.kind.closes().is_some() {
            // We want to consume delimiters until we hit a matching one, unless we never actually
            // hit a matching one.
            // - In `{{}}`, at the first `}` the stack will be `{{` and so everything will
            //   be popped.
            // - In `{[}`, the `}` will pop both `[` and `{` because the `[` is astray and should
            //   not be here.
            // - `{[}]` is a similar case to the above, but the last `]` will not pop anything
            //   because the stack is empty.
            // This mechanism can be tweaked in the future to include eg. a "weakness" mechanism,
            // where certain delimiters can be considered stronger than others, so that eg. `}`
            // can pop `(`, but `)` cannot pop `{`.
            if let Some(i) = self.delimiter_stack.iter().rposition(|&k| k == token.kind) {
                self.delimiter_stack.resize_with(i, || unreachable!());
            }
        }

        Ok(token)
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached> {
        self.inner.text_blob(is_end)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        self.inner.braced_string(left_brace_span)
    }

    fn peek_from(&mut self, context: LexicalContext, channel: Channel) -> Result<Token, LexError> {
        self.inner.peek_from(context, channel)
    }

    fn contextualize_diagnostic(&self, diagnostic: Diagnostic) -> Diagnostic {
        self.inner.contextualize_diagnostic(diagnostic)
    }
}

impl<T> ParseStream for Structured<T>
where
    T: TokenStream,
{
    fn nesting_level(&self) -> usize {
        self.delimiter_stack.len()
    }
}

impl<'a, T> Parser<'a, T>
where
    T: ParseStream,
{
    pub fn try_with_delimiter_recovery<N, C>(
        &mut self,
        inner: impl FnOnce(&mut Self) -> Result<N, ParseError>,
    ) -> Result<N, C>
    where
        C: SingleToken,
    {
        // This is called already after the opening delimiter is consumed, so it's one more than
        // we want to descend to.
        let open_nesting_level = self.tokens.nesting_level();

        match inner(self) {
            Ok(ok) => Ok(ok),
            Err(error) => {
                let mut last_token_span = None;
                // Note the use of >= here; as mentioned, we want to descend one level further
                // because at the time this function is called the opening delimiter has already
                // been consumed.
                // Also in case of closers like EndOfFile we need to check against
                // open_nesting_level being zero, so that we don't loop indefinitely.
                while self.tokens.nesting_level() >= open_nesting_level || open_nesting_level == 0 {
                    last_token_span = Some(match self.next_token() {
                        Ok(token) => {
                            if token.kind == TokenKind::EndOfFile {
                                // To prevent an infinite loop from occurring, bail early.
                                return Err(C::default_from_span(token.span));
                            }
                            token.span
                        }
                        Err(error) => error.span,
                    });
                }
                // Worst case scenario: we have to use the error span provided to us, if a token
                // consumed by `inner` happens to be a closing token and the nesting level is
                // decremented because of that.
                Err(C::default_from_span(last_token_span.unwrap_or(error.span)))
            }
        }
    }
}
