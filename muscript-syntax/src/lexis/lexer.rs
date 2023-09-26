use std::{collections::HashMap, ops::Range, rc::Rc};

use muscript_foundation::{
    errors::{Diagnostic, Label, ReplacementSuggestion},
    source::SourceFileId,
    source_arena::SourceArenaBuilder,
    span::Span,
};

use super::{
    token::{AnyToken, SourceLocation, Token, TokenId, TokenKind},
    Channel, TokenStream,
};

/// Context for lexical analysis.
///
/// In the default context multiple `>` operators use maximal munch, therefore `>>` is a single
/// token. In the type context, each `>` character produces a single `>` token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalContext {
    Default,
    Type,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    pub token_arena: SourceArenaBuilder<'a, Token>,

    pub file: SourceFileId,
    pub input: Rc<str>,
    pub position: SourceLocation,

    pub errors: HashMap<TokenId, Diagnostic<Token>>,
}

// Unnecessary casts are allowed because `SourceLocation` may not end up being a `usize` if we
// want to save some space.
#[allow(clippy::unnecessary_cast)]
impl<'a> Lexer<'a> {
    pub fn new(
        token_arena: SourceArenaBuilder<'a, Token>,
        file: SourceFileId,
        input: Rc<str>,
    ) -> Self {
        Self {
            token_arena,
            file,
            input,
            position: 0,
            errors: HashMap::new(),
        }
    }

    pub fn current_char(&self) -> Option<char> {
        if let Some(input) = self.input.get(self.position as usize..) {
            input.chars().next()
        } else {
            None
        }
    }

    pub fn advance_char(&mut self) {
        if let Some(char) = self.current_char() {
            self.position += char.len_utf8() as SourceLocation;
        }
    }

    fn range(&self, start: SourceLocation) -> Range<SourceLocation> {
        start..self.position
    }

    fn create_token(&mut self, kind: TokenKind, range: Range<SourceLocation>) -> TokenId {
        self.token_arena.push(Token {
            kind,
            source_range: range,
        })
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

    fn comment_or_division(&mut self, start: SourceLocation) -> TokenId {
        self.advance_char();
        match self.current_char() {
            Some('/') => {
                self.advance_char();
                while !matches!(self.current_char(), None | Some('\n')) {
                    self.advance_char();
                }
                // Skip the \n at the end.
                self.advance_char();
                self.create_token(TokenKind::Comment, self.range(start))
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
                            let comment_start =
                                self.create_token(TokenKind::Error, start..start + 2);
                            let _rest = self.create_token(TokenKind::Error, self.range(start + 2));
                            self.errors.insert(
                                comment_start,
                                Diagnostic::error(
                                    "block comment does not have a matching '*/' terminator",
                                )
                                .with_label(Label::primary(
                                    &Span::single(comment_start),
                                    "the comment starts here",
                                )),
                            );
                            return comment_start;
                        }
                        _ => self.advance_char(),
                    }
                }
                self.create_token(TokenKind::Comment, self.range(start))
            }
            _ => self.create_token(TokenKind::Div, self.range(start)),
        }
    }

    fn identifier(&mut self) -> TokenId {
        let start = self.position;
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.current_char() {
            self.advance_char();
        }
        self.create_token(TokenKind::Ident, self.range(start))
    }

    fn decimal_number(&mut self, start: SourceLocation) -> TokenId {
        while let Some('0'..='9') = self.current_char() {
            self.advance_char();
        }
        if self.current_char() == Some('.') {
            self.advance_char();
            while let Some('0'..='9') = self.current_char() {
                self.advance_char();
            }
            if let Some('e' | 'E') = self.current_char() {
                let exponent_start = self.position;
                self.advance_char();
                if let Some('+' | '-') = self.current_char() {
                    self.advance_char();
                }
                let _exponent_end = self.position;

                match self.one_or_more(|c| c.is_ascii_digit()) {
                    Ok(_) => {
                        if self.current_char() == Some('f') {
                            self.advance_char();
                        }
                        self.create_token(TokenKind::FloatLit, self.range(start))
                    }
                    Err(_) => {
                        let before_exponent =
                            self.create_token(TokenKind::FloatLit, start..exponent_start);
                        if self.current_char() == Some('f') {
                            self.advance_char();
                        }
                        let exponent =
                            self.create_token(TokenKind::Error, self.range(exponent_start));
                        self.errors.insert(exponent, Diagnostic::error(
                            "'e' in float literal with scientific notation must be followed by an exponent number",
                        )
                        .with_label(Label::primary(
                            &Span::single(exponent),
                            "scientific notation used here",
                        )));
                        before_exponent
                    }
                }
            } else {
                if self.current_char() == Some('f') {
                    self.advance_char();
                }
                self.create_token(TokenKind::FloatLit, self.range(start))
            }
        } else if self.current_char() == Some('f') {
            self.advance_char();
            self.create_token(TokenKind::FloatLit, self.range(start))
        } else {
            self.create_token(TokenKind::IntLit, self.range(start))
        }
    }

    fn number(&mut self, start: SourceLocation) -> TokenId {
        let literal = if self.current_char() == Some('0') {
            self.advance_char();
            if let Some('x' | 'X') = self.current_char() {
                self.advance_char();
                while let Some('0'..='9' | 'A'..='F' | 'a'..='f') = self.current_char() {
                    self.advance_char();
                }
                self.create_token(TokenKind::IntLit, self.range(start))
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
            let ident_error = self.create_token(TokenKind::Error, self.range(ident_start));
            self.errors.insert(
                ident_error,
                Diagnostic::error(
                    "number literal must not be immediately followed by an identifier",
                )
                .with_label(Label::secondary(
                    &Span::single(literal),
                    "number literal occurs here...",
                ))
                .with_label(Label::primary(
                    &Span::single(ident_error),
                    "...and is immediately followed by an identifier",
                ))
                .with_note((
                    "help: add a space between the number and the identifier",
                    ReplacementSuggestion {
                        file: self.file,
                        span: start..ident_end,
                        replacement: format!(
                            "{} {}",
                            &self.input[start as usize..ident_start as usize],
                            &self.input[ident_start as usize..ident_end as usize]
                        ),
                    },
                )),
            );
        }

        literal
    }

    fn string_char(&mut self) {
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
    }

    fn string(&mut self, start: SourceLocation) -> TokenId {
        self.advance_char();
        while self.current_char() != Some('"') {
            if self.current_char().is_none() {
                let quote = self.create_token(TokenKind::Error, start..start + 1);
                let unterminated = self.create_token(TokenKind::Error, self.range(start + 1));
                self.errors.insert(
                    unterminated,
                    Diagnostic::error("string literal does not have a closing quote `\"`")
                        .with_label(Label::primary(
                            &Span::single(quote),
                            "the string starts here",
                        )),
                );
                return unterminated;
            }
            self.string_char();
        }
        self.advance_char();
        self.create_token(TokenKind::StringLit, self.range(start))
    }

    fn name(&mut self, start: SourceLocation) -> TokenId {
        self.advance_char();
        while self.current_char() != Some('\'') {
            if self.current_char().is_none() {
                let quote = self.create_token(TokenKind::Error, start..start + 1);
                let unterminated = self.create_token(TokenKind::Error, self.range(start + 1));
                self.errors.insert(
                    unterminated,
                    Diagnostic::error("name does not have a closing quote `'`")
                        .with_label(Label::primary(&Span::single(quote), "the name starts here")),
                );
                return unterminated;
            }
            self.string_char();
        }
        self.advance_char();
        self.create_token(TokenKind::NameLit, self.range(start))
    }

    fn single_char_token(&mut self, kind: TokenKind) -> TokenId {
        let start = self.position;
        self.advance_char();
        self.create_token(kind, self.range(start))
    }

    fn single_or_double_char_token(
        &mut self,
        kind: TokenKind,
        second: char,
        second_kind: TokenKind,
    ) -> TokenId {
        let start = self.position;
        self.advance_char();
        if self.current_char() == Some(second) {
            self.advance_char();
            self.create_token(second_kind, self.range(start))
        } else {
            self.create_token(kind, self.range(start))
        }
    }
}

/// Functions used by the preprocessor.
impl<'a> Lexer<'a> {
    pub(super) fn eat_until_line_feed(&mut self) {
        while !matches!(self.current_char(), Some('\n') | None) {
            self.advance_char();
        }
        self.advance_char(); // Advance past the line feed too.
    }
}

impl<'a> TokenStream for Lexer<'a> {
    fn next_any(&mut self, context: LexicalContext) -> AnyToken {
        self.skip_whitespace();

        let start = self.position;

        let id = if let Some(char) = self.current_char() {
            match char {
                '/' => self.comment_or_division(start),
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
                '0'..='9' => self.number(start),
                '"' => self.string(start),
                '\'' => self.name(start),
                '+' => self.single_or_double_char_token(TokenKind::Add, '+', TokenKind::Inc),
                '-' => self.single_or_double_char_token(TokenKind::Sub, '-', TokenKind::Dec),
                '*' => self.single_or_double_char_token(TokenKind::Mul, '*', TokenKind::Pow),
                '%' => self.single_char_token(TokenKind::Rem),
                '<' => {
                    self.advance_char();
                    match self.current_char() {
                        Some('<') => {
                            self.advance_char();
                            self.create_token(TokenKind::ShiftLeft, self.range(start))
                        }
                        Some('=') => {
                            self.advance_char();
                            self.create_token(TokenKind::LessEqual, self.range(start))
                        }
                        _ => self.create_token(TokenKind::Less, self.range(start)),
                    }
                }
                '>' => {
                    self.advance_char();
                    match self.current_char() {
                        Some('>') if context != LexicalContext::Type => {
                            self.advance_char();
                            if self.current_char() == Some('>') {
                                self.advance_char();
                                self.create_token(TokenKind::TripleShiftRight, self.range(start))
                            } else {
                                self.create_token(TokenKind::ShiftRight, self.range(start))
                            }
                        }
                        Some('=') => {
                            self.advance_char();
                            self.create_token(TokenKind::GreaterEqual, self.range(start))
                        }
                        _ => self.create_token(TokenKind::Greater, self.range(start)),
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
                        self.decimal_number(start)
                    } else {
                        self.create_token(TokenKind::Dot, self.range(start))
                    }
                }
                ',' => self.single_char_token(TokenKind::Comma),
                ';' => self.single_char_token(TokenKind::Semi),
                '#' => self.single_char_token(TokenKind::Hash),
                '`' => self.single_char_token(TokenKind::Accent),
                '\\' => self.single_char_token(TokenKind::Backslash),
                unknown => {
                    let unrecognized_character =
                        self.create_token(TokenKind::Error, self.range(start));

                    self.errors.insert(
                        unrecognized_character,
                        Diagnostic::error(format!("unrecognized character: {unknown:?}"))
                            .with_label(Label::primary(
                                &Span::single(unrecognized_character),
                                "this character is not valid syntax",
                            )),
                    );
                    unrecognized_character
                }
            }
        } else {
            self.create_token(TokenKind::EndOfFile, self.range(start))
        };

        let kind = self.token_arena.arena().element(id).kind;
        AnyToken { kind, id }
    }

    fn peek_from(&mut self, context: LexicalContext, channel: Channel) -> AnyToken {
        let position = self.position;
        let result = self.next_from(context, channel);
        self.position = position;
        result
    }
}
