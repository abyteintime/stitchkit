use std::rc::Rc;

use muscript_foundation::{
    errors::{Diagnostic, Label, ReplacementSuggestion},
    source::{SourceFileId, Span},
};

use super::{
    token::{Token, TokenKind},
    Channel, EofReached, LexError, TokenStream,
};

#[derive(Debug)]
pub struct Lexer {
    pub file: SourceFileId,
    pub input: Rc<str>,
    pub position: usize,
}

impl Lexer {
    pub fn new(file: SourceFileId, input: Rc<str>) -> Self {
        Self {
            file,
            input,
            position: 0,
        }
    }

    pub fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    pub fn advance_char(&mut self) {
        if let Some(char) = self.current_char() {
            self.position += char.len_utf8();
        }
    }

    fn span(&self, start: usize) -> Span {
        Span::from(start..self.position)
    }

    fn span_with_len(&self, start: usize, len: usize) -> Span {
        let len = self.input[start..]
            .char_indices()
            .skip(len)
            .map(|(index, _)| index)
            .next()
            .unwrap_or(self.input.len());
        Span::from(start..start + len)
    }

    fn one_or_more(&mut self, mut test: impl Fn(char) -> bool) -> Result<(), ()> {
        if !self.current_char().map(&mut test).unwrap_or(false) {
            return Err(());
        }
        while self.current_char().map(&mut test).unwrap_or(false) {
            self.advance_char();
        }
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ' | '\t' | '\r' | '\n') = self.current_char() {
            self.advance_char();
        }
    }

    fn comment_or_division(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.advance_char();
        match self.current_char() {
            Some('/') => {
                self.advance_char();
                while !matches!(self.current_char(), None | Some('\n')) {
                    self.advance_char();
                }
                // Skip the \n at the end.
                self.advance_char();
                Ok(TokenKind::Comment)
            }
            Some('*') => {
                self.advance_char();
                let mut nesting = 1;
                while nesting > 0 {
                    match self.current_char() {
                        Some('*') => {
                            self.advance_char();
                            if self.current_char() == Some('/') {
                                nesting -= 1;
                                self.advance_char();
                            }
                        }
                        Some('/') => {
                            self.advance_char();
                            if self.current_char() == Some('*') {
                                nesting += 1;
                                self.advance_char();
                            }
                        }
                        None => {
                            return Err(LexError::new(
                                self.span(start),
                                Diagnostic::error(
                                    self.file,
                                    "block comment does not have a matching '*/' terminator",
                                )
                                .with_label(Label::primary(
                                    self.span_with_len(start, 2),
                                    "the comment starts here",
                                )),
                            ))
                        }
                        _ => self.advance_char(),
                    }
                }
                Ok(TokenKind::Comment)
            }
            _ => Ok(TokenKind::Div),
        }
    }

    fn identifier(&mut self) -> TokenKind {
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.current_char() {
            self.advance_char();
        }
        TokenKind::Ident
    }

    fn decimal_number(&mut self, start: usize) -> Result<TokenKind, LexError> {
        while let Some('0'..='9') = self.current_char() {
            self.advance_char();
        }
        if self.current_char() == Some('.') {
            self.advance_char();
            while let Some('0'..='9') = self.current_char() {
                self.advance_char();
            }
            let result = if let Some('e' | 'E') = self.current_char() {
                let exponent_start = self.position;
                self.advance_char();
                if let Some('+' | '-') = self.current_char() {
                    self.advance_char();
                }
                let exponent_end = self.position;
                self.one_or_more(|c| c.is_ascii_digit()).map_err(|_| {
                    LexError::new(
                        self.span(start),
                        Diagnostic::error(
                            self.file,
                            "'e' in float literal with scientific notation must be followed by an exponent number",
                        )
                        .with_label(Label::primary(
                            Span::from(exponent_start..exponent_end),
                            "scientific notation used here",
                        )),
                    )
                })
            } else {
                Ok(())
            }
            .map(|_| TokenKind::FloatLit);
            // NOTE: Even in case of error above, we want to continue reading to skip the possible
            // f suffix so that the parser doesn't have to deal with a stray identifier.
            if self.current_char() == Some('f') {
                self.advance_char();
            }
            result
        } else if self.current_char() == Some('f') {
            self.advance_char();
            Ok(TokenKind::FloatLit)
        } else {
            Ok(TokenKind::IntLit)
        }
    }

    fn number(&mut self, start: usize) -> Result<TokenKind, LexError> {
        let result = if self.current_char() == Some('0') {
            self.advance_char();
            if let Some('x' | 'X') = self.current_char() {
                self.advance_char();
                while let Some('0'..='9' | 'A'..='F' | 'a'..='f') = self.current_char() {
                    self.advance_char();
                }
                Ok(TokenKind::IntLit)
            } else {
                // Again, we don't want to early-out here to not leave the parser with a
                // stray identifier.
                self.decimal_number(start)
            }
        } else {
            self.decimal_number(start)
        };

        if let Some('A'..='Z' | 'a'..='z' | '_') = self.current_char() {
            let ident_start = self.position;
            self.identifier();
            let ident_end = self.position;
            return Err(LexError::new(
                self.span(start),
                Diagnostic::error(
                    self.file,
                    "number literal must not be immediately followed by an identifier",
                )
                .with_label(Label::secondary(
                    Span::from(start..ident_start),
                    "number literal occurs here...",
                ))
                .with_label(Label::primary(
                    Span::from(ident_start..ident_end),
                    "...and is immediately followed by an identifier",
                ))
                .with_note((
                    "help: add a space between the number and the identifier",
                    ReplacementSuggestion {
                        span: Span::from(start..ident_end),
                        replacement: format!(
                            "{} {}",
                            &self.input[start..ident_start],
                            &self.input[ident_start..ident_end]
                        ),
                    },
                )),
            ));
        }

        result
    }

    fn string_char(&mut self) -> Result<(), LexError> {
        // The lexer doesn't do any parsing of escape sequences. For the lexer only really \"
        // counts, so that it knows where the string ends.
        match self.current_char() {
            Some('\\') => {
                self.advance_char();
                // Advance over the escaped character.
                self.advance_char();
            }
            _ => self.advance_char(),
        }
        Ok(())
    }

    fn string(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.advance_char();
        while self.current_char() != Some('"') {
            if self.current_char().is_none() {
                return Err(LexError::new(
                    self.span(start),
                    Diagnostic::error(
                        self.file,
                        "string literal does not have a closing quote '\"'",
                    )
                    .with_label(Label::primary(
                        self.span_with_len(start, 1),
                        "the string starts here",
                    )),
                ));
            }
            self.string_char()?;
        }
        self.advance_char();
        Ok(TokenKind::StringLit)
    }

    fn name(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.advance_char();
        while self.current_char() != Some('\'') {
            if self.current_char().is_none() {
                return Err(LexError::new(
                    self.span(start),
                    Diagnostic::error(self.file, "name does not have a closing quote '\"'")
                        .with_label(Label::primary(
                            self.span_with_len(start, 1),
                            "the name starts here",
                        )),
                ));
            }
            self.string_char()?;
        }
        self.advance_char();
        Ok(TokenKind::NameLit)
    }

    fn single_char_token(&mut self, kind: TokenKind) -> TokenKind {
        self.advance_char();
        kind
    }

    fn single_or_double_char_token(
        &mut self,
        kind: TokenKind,
        second: char,
        second_kind: TokenKind,
    ) -> TokenKind {
        self.advance_char();
        if self.current_char() == Some(second) {
            self.advance_char();
            second_kind
        } else {
            kind
        }
    }
}

/// Functions used by the preprocessor.
impl Lexer {
    pub(super) fn eat_until_line_feed(&mut self) {
        while !matches!(self.current_char(), Some('\n') | None) {
            self.advance_char();
        }
        self.advance_char(); // Advance past the line feed too.
    }
}

impl TokenStream for Lexer {
    fn next_any(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let start = self.position;

        let kind = if let Some(char) = self.current_char() {
            match char {
                '/' => self.comment_or_division(start)?,
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
                '0'..='9' => self.number(start)?,
                '"' => self.string(start)?,
                '\'' => self.name(start)?,
                '+' => self.single_or_double_char_token(TokenKind::Add, '+', TokenKind::Inc),
                '-' => self.single_or_double_char_token(TokenKind::Sub, '-', TokenKind::Dec),
                '*' => self.single_or_double_char_token(TokenKind::Mul, '*', TokenKind::Pow),
                '%' => self.single_char_token(TokenKind::Rem),
                '<' => {
                    self.advance_char();
                    match self.current_char() {
                        Some('<') => {
                            self.advance_char();
                            TokenKind::ShiftLeft
                        }
                        Some('=') => {
                            self.advance_char();
                            TokenKind::LessEqual
                        }
                        _ => TokenKind::Less,
                    }
                }
                '>' => {
                    self.advance_char();
                    match self.current_char() {
                        Some('>') => {
                            self.advance_char();
                            if self.current_char() == Some('>') {
                                self.advance_char();
                                TokenKind::TripleShiftRight
                            } else {
                                TokenKind::ShiftRight
                            }
                        }
                        Some('=') => {
                            self.advance_char();
                            TokenKind::GreaterEqual
                        }
                        _ => TokenKind::Greater,
                    }
                }
                '&' => self.single_or_double_char_token(TokenKind::BitAnd, '&', TokenKind::And),
                '|' => self.single_or_double_char_token(TokenKind::BitOr, '|', TokenKind::Or),
                '^' => self.single_or_double_char_token(TokenKind::BitXor, '^', TokenKind::Xor),
                '$' => self.single_char_token(TokenKind::Dollar),
                '@' => self.single_char_token(TokenKind::At),
                ':' => self.single_char_token(TokenKind::Colon),
                '?' => self.single_char_token(TokenKind::Question),
                '!' => self.single_or_double_char_token(TokenKind::Not, '=', TokenKind::NotEqual),
                '=' => self.single_or_double_char_token(TokenKind::Assign, '=', TokenKind::Equal),
                '~' => {
                    self.single_or_double_char_token(TokenKind::BitNot, '=', TokenKind::ApproxEqual)
                }
                '(' => self.single_char_token(TokenKind::LeftParen),
                ')' => self.single_char_token(TokenKind::RightParen),
                '[' => self.single_char_token(TokenKind::LeftBracket),
                ']' => self.single_char_token(TokenKind::RightBracket),
                '{' => self.single_char_token(TokenKind::LeftBrace),
                '}' => self.single_char_token(TokenKind::RightBrace),
                '.' => {
                    self.advance_char();
                    if let Some('0'..='9') = self.current_char() {
                        // Hack: need to advance the position back to the `.` so that
                        // decimal_number can pick it up properly and not allow `.0.0`, which would
                        // be invalid syntax.
                        self.position -= 1;
                        self.decimal_number(start)?
                    } else {
                        TokenKind::Dot
                    }
                }
                ',' => self.single_char_token(TokenKind::Comma),
                ';' => self.single_char_token(TokenKind::Semi),
                '#' => self.single_char_token(TokenKind::Hash),
                '`' => self.single_char_token(TokenKind::Accent),
                '\\' => self.single_char_token(TokenKind::Backslash),
                unknown => {
                    return Err(LexError::new(
                        self.span(start),
                        Diagnostic::error(
                            self.file,
                            format!("unrecognized character: {unknown:?}"),
                        )
                        .with_label(Label::primary(
                            self.span(start),
                            "this character is not valid syntax",
                        )),
                    ))
                }
            }
        } else {
            TokenKind::EndOfFile
        };

        let end = self.position;
        Ok(Token {
            kind,
            span: Span::from(start..end),
        })
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached> {
        let start = self.position;
        loop {
            if let Some(char) = self.current_char() {
                if is_end(char) {
                    break;
                }
                self.advance_char();
            } else {
                return Err(EofReached);
            }
        }
        let end = self.position;
        Ok(Span::from(start..end))
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        let start = self.position;

        let mut nesting = 1;
        while nesting > 0 {
            match self.current_char() {
                Some('{') => {
                    nesting += 1;
                    self.advance_char();
                }
                Some('}') => {
                    nesting -= 1;
                    if nesting != 0 {
                        // If nesting is zero, we don't wanna consume the right brace
                        // because the parser turns it into a token.
                        self.advance_char();
                    }
                }
                None => {
                    return Err(LexError::new(
                        Span::from(start..self.position),
                        Diagnostic::error(
                            self.file,
                            "braced string is missing its right brace `}`",
                        )
                        .with_label(Label::primary(
                            left_brace_span,
                            "the braced string starts here",
                        ))
                        .with_note("note: braced strings may nest `{hello {world}}`"),
                    ))
                }
                _ => self.advance_char(),
            }
        }

        let end = self.position;
        Ok(Span::from(start..end))
    }

    fn peek_from(&mut self, channel: Channel) -> Result<Token, LexError> {
        let position = self.position;
        let result = self.next_from(channel);
        self.position = position;
        result
    }
}
