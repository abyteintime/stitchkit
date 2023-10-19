use muscript_lexer::{token::TokenKind, token_stream::TokenStream};

use crate::{token::SingleToken, ParseError, Parser};

impl<'a, T> Parser<'a, T>
where
    T: TokenStream,
{
    pub fn nesting_level(&self) -> usize {
        self.delimiter_stack.len()
    }

    pub fn try_with_delimiter_recovery<N, C>(
        &mut self,
        inner: impl FnOnce(&mut Self) -> Result<N, ParseError>,
    ) -> Result<N, C>
    where
        C: SingleToken,
    {
        // This is called already after the opening delimiter is consumed, so it's one more than
        // we want to descend to.
        let open_nesting_level = self.nesting_level();

        match inner(self) {
            Ok(ok) => Ok(ok),
            Err(error) => {
                let mut last_token_span = None;
                // Note the use of >= here; as mentioned, we want to descend one level further
                // because at the time this function is called the opening delimiter has already
                // been consumed.
                // Also in case of closers like EndOfFile we need to check against
                // open_nesting_level being zero, so that we don't loop indefinitely.
                while self.nesting_level() >= open_nesting_level || open_nesting_level == 0 {
                    last_token_span = Some({
                        let token = self.next_token();
                        if token.kind == TokenKind::EndOfFile {
                            // To prevent an infinite loop from occurring, bail early.
                            return Err(C::default_from_id(token.id));
                        }
                        token.id
                    });
                }
                // Worst case scenario: we have to use the error span provided to us, if a token
                // consumed by `inner` happens to be a closing token and the nesting level is
                // decremented because of that.
                Err(C::default_from_id(
                    last_token_span
                        .or(error.span.start())
                        .or(error.span.end())
                        .expect("parse error span must not be empty"),
                ))
            }
        }
    }
}
