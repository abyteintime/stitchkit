use std::num::{IntErrorKind, ParseIntError};

use muscript_foundation::errors::{Diagnostic, DiagnosticSink, Label};
use muscript_lexer::sources::LexedSources;

use crate::diagnostics::notes;

use super::{FloatLit, IntLit, NameLit, StringLit, Token};

// NOTE: Currently int parsing is not ideal, because the corner case of -0x80000000 is not handled
// correctly, as the negative sign is not part of the integer literal.
impl IntLit {
    fn map_parse_error(
        &self,
        result: Result<i32, ParseIntError>,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> i32 {
        match result {
            Ok(num) => num,
            Err(error) => match error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    diagnostics.emit(
                        Diagnostic::error("integer does not fit within 32 bits")
                            .with_label(Label::primary(self, ""))
                            .with_note(indoc::indoc! {"
                                note: UnrealScript integers are 32 bit;
                                      this means their values are in the range [-2147483648, 2147483647]
                                      (or [-0x80000000, 0x7FFFFFFF] in hex)
                            "}),
                    );
                    0
                }
                _ => {
                    diagnostics.emit(
                        Diagnostic::bug(format!("unexpected error when parsing integer: {error}"))
                            .with_label(Label::primary(self, ""))
                            .with_note(notes::PARSER_BUG),
                    );
                    0
                }
            },
        }
    }

    pub fn parse(
        &self,
        sources: &LexedSources<'_>,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> i32 {
        let int = sources.source(self);
        self.map_parse_error(
            if int.starts_with("0x") || int.starts_with("0X") {
                i32::from_str_radix(&int[2..], 16)
            } else {
                int.parse()
            },
            diagnostics,
        )
    }
}

impl FloatLit {
    pub fn parse(
        &self,
        sources: &LexedSources<'_>,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> f32 {
        let float = sources.source(self);
        match float.strip_suffix('f').unwrap_or(float).parse() {
            Ok(f) => f,
            Err(error) => {
                diagnostics.emit(
                    Diagnostic::bug(format!("unexpected error when parsing float: {error}"))
                        .with_label(Label::primary(self, ""))
                        .with_note(notes::PARSER_BUG),
                );
                0.0
            }
        }
    }
}

impl StringLit {
    pub fn parse(
        &self,
        sources: &LexedSources<'_>,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> String {
        let string = sources.source(self);
        let string = &string[1..string.len() - 1];

        let mut result = String::with_capacity(string.len());
        let mut iter = string.char_indices();
        loop {
            match iter.next() {
                Some((_start, '\\')) => match iter.next() {
                    Some((_, 'n')) => result.push('\n'),
                    Some((_, other)) => diagnostics.emit(
                        Diagnostic::error(format!("invalid escape sequence: `\\{other}`"))
                            // TODO: As a result of refactoring the lexer, it is now impossible to
                            // subslice the string in an error message. Therefore StringLit tokens
                            // should probably be split up into StringBegin, StringData, StringEscape,
                            // and StringEnd.
                            .with_label(Label::primary(self, "")),
                        // TODO: List escape sequences here.
                        // .with_note("note: supported escape sequences include: "),
                    ),
                    None => unreachable!("\\\" is an escape sequence that continues the string"),
                },
                Some((_, c)) => result.push(c),
                None => break,
            }
        }
        result
    }
}

impl NameLit {
    pub fn parse<'a>(&self, sources: &'a LexedSources<'a>) -> &'a str {
        let name = sources.source(self);
        &name[1..name.len() - 1]
    }
}
