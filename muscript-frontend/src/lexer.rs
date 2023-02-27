use muscript_foundation::{
    errors::{Diagnostic, Label, ReplacementSuggestion, Span},
    source::SourceFileId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Comment,

    Ident,

    None,
    True,
    False,

    Int,
    IntHex,
    Float,
    String,
    Name,

    Add,              // +
    Sub,              // -
    Mul,              // *
    Div,              // /
    Rem,              // %
    Pow,              // **
    KDot,             // dot
    Cross,            // cross
    Dollar,           // $
    At,               // @
    Colon,            // :
    Question,         // ?
    ShiftLeft,        // <<
    ShiftRight,       // >>
    TripleShiftRight, // >>>
    BitNot,           // ~
    BitAnd,           // &
    BitOr,            // |
    BitXor,           // ^
    Not,              // !
    Equal,            // ==
    NotEqual,         // !=
    ApproxEqual,      // ~=
    Less,             // <
    Greater,          // >
    LessEqual,        // <=
    GreaterEqual,     // >=
    And,              // &&
    Or,               // ||
    Xor,              // ^^
    Inc,              // ++
    Dec,              // --
    Assign,           // =

    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Dot,          // .
    Comma,        // ,
    Semicolon,    // ;
    Hash,         // #
    Accent,       // `
    Backslash,    // \

    EndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    pub file: SourceFileId,
    pub input: &'a str,
    pub position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(file: SourceFileId, input: &'a str) -> Self {
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
        Span(start..self.position)
    }

    fn span_with_len(&self, start: usize, len: usize) -> Span {
        let len = self.input[start..]
            .char_indices()
            .skip(len)
            .map(|(index, _)| index)
            .next()
            .unwrap_or(self.input.len());
        Span(start..start + len)
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
                            return Err(LexError(
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

    fn decimal_number(&mut self) -> Result<TokenKind, LexError> {
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
                let exponent_end = self.position;
                self.one_or_more(|c| c.is_ascii_digit()).map_err(|_| {
                    LexError(
                        Diagnostic::error(
                            self.file,
                            "'e' in float literal with scientific notation must be followed by an exponent number",
                        )
                        .with_label(Label::primary(
                            Span(exponent_start..exponent_end),
                            "scientific notation used here",
                        )),
                    )
                })?;
            }
            if self.current_char() == Some('f') {
                self.advance_char();
                return Ok(TokenKind::Float);
            }
            Ok(TokenKind::Float)
        } else if self.current_char() == Some('f') {
            self.advance_char();
            Ok(TokenKind::Float)
        } else {
            Ok(TokenKind::Int)
        }
    }

    fn number(&mut self, start: usize) -> Result<TokenKind, LexError> {
        let kind = if self.current_char() == Some('0') {
            self.advance_char();
            if let Some('x' | 'X') = self.current_char() {
                self.advance_char();
                while let Some('0'..='9' | 'A'..='F' | 'a'..='f') = self.current_char() {
                    self.advance_char();
                }
                TokenKind::IntHex
            } else {
                self.decimal_number()?
            }
        } else {
            self.decimal_number()?
        };

        if let Some('A'..='Z' | 'a'..='z' | '_') = self.current_char() {
            let ident_start = self.position;
            self.identifier();
            let ident_end = self.position;
            return Err(LexError(
                Diagnostic::error(
                    self.file,
                    "number literal must not be immediately followed by an identifier",
                )
                .with_label(Label::secondary(
                    Span(start..ident_start),
                    "number literal occurs here...",
                ))
                .with_label(Label::primary(
                    Span(ident_start..ident_end),
                    "...and is immediately followed by an identifier",
                ))
                .with_note((
                    "help: add a space between the number and the identifier",
                    ReplacementSuggestion {
                        span: Span(start..ident_end),
                        replacement: format!(
                            "{} {}",
                            &self.input[start..ident_start],
                            &self.input[ident_start..ident_end]
                        ),
                    },
                )),
            ));
        }

        Ok(kind)
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
                dbg!(self.span_with_len(start, 1));
                return Err(LexError(
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
        Ok(TokenKind::String)
    }

    fn name(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.advance_char();
        while self.current_char() != Some('\'') {
            if self.current_char().is_none() {
                dbg!(self.span_with_len(start, 1));
                return Err(LexError(
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
        Ok(TokenKind::Name)
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

    pub fn next_token_include_comments(&mut self) -> Result<Token, LexError> {
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
                '<' => self.single_or_double_char_token(TokenKind::Less, '<', TokenKind::ShiftLeft),
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
                '.' => self.single_char_token(TokenKind::Dot),
                ',' => self.single_char_token(TokenKind::Comma),
                ';' => self.single_char_token(TokenKind::Semicolon),
                '#' => self.single_char_token(TokenKind::Hash),
                '`' => self.single_char_token(TokenKind::Accent),
                '\\' => self.single_char_token(TokenKind::Backslash),
                unknown => {
                    return Err(LexError(
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
            span: Span(start..end),
        })
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.next_token_include_comments()?;
            if token.kind != TokenKind::Comment {
                return Ok(token);
            }
        }
    }
}

pub struct LexError(pub Diagnostic);

impl From<LexError> for Vec<Diagnostic> {
    fn from(value: LexError) -> Self {
        vec![value.0]
    }
}
