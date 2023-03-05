use muscript_foundation::source::Span;

use super::{
    token::{Token, TokenKind},
    LexError,
};

pub trait TokenStream {
    type Position;

    fn position(&self) -> Self::Position;
    fn seek(&mut self, to: Self::Position);

    fn next_include_comments(&mut self) -> Result<Token, LexError>;

    fn next(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.next_include_comments()?;
            if token.kind != TokenKind::Comment {
                return Ok(token);
            }
        }
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError>;

    fn peek_include_comments(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        let result = self.next_include_comments();
        self.seek(position);
        result
    }

    fn peek(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        let result = self.next();
        self.seek(position);
        result
    }
}

impl<T> TokenStream for &mut T
where
    T: TokenStream,
{
    type Position = T::Position;

    fn position(&self) -> Self::Position {
        <T as TokenStream>::position(self)
    }

    fn seek(&mut self, to: Self::Position) {
        <T as TokenStream>::seek(self, to)
    }

    fn next_include_comments(&mut self) -> Result<Token, LexError> {
        <T as TokenStream>::next_include_comments(self)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        <T as TokenStream>::braced_string(self, left_brace_span)
    }
}
