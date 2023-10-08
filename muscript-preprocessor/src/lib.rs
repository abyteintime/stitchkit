pub mod sliced_tokens;

use std::collections::HashMap;

use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
};
use muscript_lexer::{
    sources::LexedSources,
    token::{AnyToken, Token, TokenKind, TokenSpan},
    token_stream::TokenStream,
};
use sliced_tokens::SlicedTokens;

use crate::sliced_tokens::TokenSlice;

/// A map of definitions. These may be constructed externally, to provide the preprocessor with
/// symbols such as `FINAL_RELEASE`.
#[derive(Debug, Clone, Default)]
pub struct Definitions {
    pub map: HashMap<CaseInsensitive<String>, Definition>,
}

/// A single preprocessor definition.
#[derive(Debug, Clone)]
pub struct Definition {
    pub source_span: TokenSpan,
    pub parameters: Option<Vec<String>>,
}

/// Preprocessor that sits between the lexer and the parser.
///
/// Note that the MuScript preprocessor does not strive for 100% compatibility with the UnrealScript
/// preprocessor, largely because that would require sacrificing a lot of MuScript's error reporting
/// infrastructure. Therefore this preprocessor only really supports features on a best effort
/// basis; only enough features are supported to compile the engine successfully.
///
/// In general MuScript's improved ergonomics should be preferred over abusing the preprocessor
/// as is typical in UnrealScript programming.
pub struct Preprocessor<'a, T> {
    pub definitions: &'a mut Definitions,
    sources: LexedSources<'a>,
    tokens: T,
    diagnostics: &'a mut dyn DiagnosticSink<Token>,
    out_tokens: &'a mut SlicedTokens,
    current_span: TokenSpan,
}

impl<'a, T> Preprocessor<'a, T> {
    pub fn new(
        definitions: &'a mut Definitions,
        sources: LexedSources<'a>,
        in_tokens: T,
        out_tokens: &'a mut SlicedTokens,
        diagnostics: &'a mut dyn DiagnosticSink<Token>,
    ) -> Self {
        Self {
            definitions,
            sources,
            tokens: in_tokens,
            diagnostics,
            out_tokens,
            current_span: TokenSpan::Empty,
        }
    }

    fn flush(&mut self) {
        if let TokenSpan::Spanning { start, end } = self.current_span {
            self.out_tokens.push(TokenSlice::Span { start, end });
            self.current_span = TokenSpan::Empty;
        }
    }
}

/// # Parsing primitives
impl<'a, T> Preprocessor<'a, T>
where
    T: TokenStream,
{
    fn expect_token(
        &mut self,
        kind: TokenKind,
        make_error: impl FnOnce(AnyToken) -> Diagnostic<Token>,
    ) -> Option<AnyToken> {
        let token = self.tokens.next();
        if token.kind == kind {
            Some(token)
        } else {
            self.diagnostics.emit(make_error(token));
            None
        }
    }

    fn parse_comma_separated<E>(
        &mut self,
        left_paren: AnyToken,
        mut parse: impl FnMut(&mut Self) -> E,
    ) -> (Vec<E>, AnyToken) {
        // This is more or less just duplicating the logic in parse_separated_list.
        // It ain't nice, but coaxing the parser to do preprocessor work ain't it either.

        // Unfortunately the diagnostics here aren't as great, mainly for simplicity's sake.
        // You shouldn't be abusing the preprocessor too hard anyways. MuScript has it for
        // compatibility reasons.
        let mut elements = vec![];
        let close = loop {
            let token = self.tokens.peek();
            match token.kind {
                TokenKind::EndOfFile => {
                    self.diagnostics
                        .emit(Diagnostic::error("mismatched parenthesis").with_label(
                            Label::primary(&left_paren, "this `(` does not have a matching `)`"),
                        ));
                    return (elements, self.tokens.next());
                }
                TokenKind::RightParen => {
                    break self.tokens.next();
                }
                _ => (),
            }
            elements.push(parse(self));
            let next_token = self.tokens.next();
            match next_token.kind {
                TokenKind::Comma => (),
                TokenKind::RightParen => {
                    break next_token;
                }
                _ => {
                    self.diagnostics.emit(
                        Diagnostic::error("`,` or `)` expected")
                            .with_label(Label::primary(&next_token, "")),
                    );
                    return (elements, next_token);
                }
            }
        };

        (elements, close)
    }
}

/// # The parser
impl<'a, T> Preprocessor<'a, T>
where
    T: TokenStream,
{
    fn parse_macro_invocation(&mut self, accent: AnyToken) {
        let Some(macro_name_ident) = self.expect_token(TokenKind::Ident, |token| {
            Diagnostic::error("macro name expected").with_label(Label::primary(
                &token,
                format!("macro name expected, but got {}", token.kind.name()),
            ))
        }) else {
            return;
        };

        let macro_name = self.sources.source(&macro_name_ident);
        match macro_name {
            _ if macro_name.eq_ignore_ascii_case("define") => self.parse_define(),
            _ if macro_name.eq_ignore_ascii_case("undefine") => self.parse_undefine(),
            _ if macro_name.eq_ignore_ascii_case("isdefined") => {
                self.parse_isdefined(accent, false)
            }
            _ if macro_name.eq_ignore_ascii_case("notdefined") => {
                self.parse_isdefined(accent, true)
            }
            _ if macro_name.eq_ignore_ascii_case("include") => self.parse_include(macro_name_ident),
            _ => self.diagnostics.emit(
                Diagnostic::bug("custom macro invocations are not yet implemented")
                    .with_label(Label::primary(&macro_name_ident, "")),
            ),
        }
    }

    fn parse_define(&mut self) {
        let Some(macro_name_ident) = self.expect_token(TokenKind::Ident, |token| {
            Diagnostic::error("new macro name expected")
                .with_label(Label::primary(&token, "identifier expected here"))
        }) else {
            return;
        };

        let parameters = if self.tokens.peek().kind == TokenKind::LeftParen {
            let open = self.tokens.next();
            let (parameters, _) = self.parse_comma_separated(open, |preprocessor| {
                let parameter = preprocessor.expect_token(TokenKind::Ident, |token| {
                    Diagnostic::error("macro argument name expected")
                        .with_label(Label::primary(&token, "identifier expected here"))
                });
                let name = preprocessor.sources.source(&parameter);
                name.to_owned()
            });
            Some(parameters)
        } else {
            None
        };

        let mut span = TokenSpan::Empty;
        loop {
            let token = self.tokens.next();
            match token.kind {
                TokenKind::Backslash => {
                    self.expect_token(TokenKind::NewLine, |non_newline_token| {
                        Diagnostic::error("newline expected after backslash `\\`")
                            .with_label(Label::secondary(&token, "this backslash indicates the macro should carry over to the next line"))
                            .with_label(Label::primary(&non_newline_token, "this token is where a newline was expected"))
                    });
                }
                TokenKind::NewLine | TokenKind::EndOfFile => break,
                _ => (),
            }
            span = span.join(&TokenSpan::single(token.id));
        }

        let macro_name = self.sources.source(&macro_name_ident);
        if let Some(_old) = self.definitions.map.insert(
            CaseInsensitive::new(String::from(macro_name)),
            Definition {
                source_span: span,
                parameters,
            },
        ) {
            // TODO: Warning on redefinition of macro?
        }
    }

    fn parse_undefine(&mut self) {
        const NOTE: &str = indoc! {"
            note: `undefine expects a parameter containing the macro name, like:
                      `undefine(EXAMPLE_MACRO)
        "};

        let Some(_) = self.expect_token(TokenKind::LeftParen, |token| {
            Diagnostic::error("`(` expected")
                .with_label(Label::primary(&token, "`(` expected here"))
                .with_note(NOTE)
        }) else {
            return;
        };
        let Some(macro_name) = self.expect_token(TokenKind::Ident, |token| {
            Diagnostic::error("missing name of macro to undefine")
                .with_label(Label::primary(&token, "identifier expected here"))
                .with_note(NOTE)
        }) else {
            return;
        };
        let Some(_) = self.expect_token(TokenKind::RightParen, |token| {
            Diagnostic::error("`)` expected to close `undefine invocation")
                .with_label(Label::primary(&token, "`)` expected here"))
                .with_note(NOTE)
        }) else {
            return;
        };

        let macro_name = self.sources.source(&macro_name);
        if self
            .definitions
            .map
            .remove(CaseInsensitive::new_ref(macro_name))
            .is_none()
        {
            // TODO: Warning when a macro that is never defined is undefined?
        }
    }

    fn parse_isdefined(&mut self, accent: AnyToken, not: bool) {
        const NOTE: [&str; 2] = [
            indoc! {"
                note: `isdefined expects a parameter containing the macro name, like:
                          `isdefined(EXAMPLE_MACRO)
            "},
            indoc! {"
                note: `notdefined expects a parameter containing the macro name, like:
                          `notdefined(EXAMPLE_MACRO)
            "},
        ];

        let Some(_) = self.expect_token(TokenKind::LeftParen, |token| {
            Diagnostic::error("`(` expected")
                .with_label(Label::primary(&token, "`(` expected here"))
                .with_note(NOTE[not as usize])
        }) else {
            return;
        };
        let Some(macro_name_ident) = self.expect_token(TokenKind::Ident, |token| {
            Diagnostic::error("missing name of macro to check")
                .with_label(Label::primary(&token, "identifier expected here"))
                .with_note(NOTE[not as usize])
        }) else {
            return;
        };
        let Some(right_paren) = self.expect_token(TokenKind::RightParen, |token| {
            Diagnostic::error(
                [
                    "`)` expected to close `isdefined invocation",
                    "`)` expected to close `notdefined invocation",
                ][not as usize],
            )
            .with_label(Label::primary(&token, "`)` expected here"))
            .with_note(NOTE[not as usize])
        }) else {
            return;
        };

        let macro_name = self.sources.source(&macro_name_ident);
        let is_defined = self
            .definitions
            .map
            .contains_key(CaseInsensitive::new_ref(macro_name));
        let emit_non_empty = if not { !is_defined } else { is_defined };

        if emit_non_empty {
            self.out_tokens.push(TokenSlice::Span {
                start: accent.id,
                end: right_paren.id,
            });
        } else {
            self.out_tokens
                .push(TokenSlice::Empty { source: accent.id });
        }
    }

    fn parse_include(&mut self, include: AnyToken) {
        self.diagnostics.emit(
            Diagnostic::warning("use of `include preprocessor directive")
                .with_label(Label::primary(&include, ""))
                .with_note("note: MuScript ignores `include directives because it processes files in the correct order automatically"),
        );

        let Some(left_paren) = self.expect_token(TokenKind::LeftParen, |token| {
            Diagnostic::error("`(` expected")
                .with_label(Label::primary(&token, "`(` expected here"))
        }) else {
            return;
        };

        loop {
            let token = self.tokens.peek();
            match token.kind {
                TokenKind::RightParen => break,
                TokenKind::EndOfFile => {
                    self.diagnostics.emit(
                        Diagnostic::error("missing `)` to close `include path").with_label(
                            Label::primary(&left_paren, "this `(` does not have a matching `)`"),
                        ),
                    );
                    return;
                }
                _ => {
                    self.tokens.next();
                }
            }
        }

        let Some(_right_paren) = self.expect_token(TokenKind::RightParen, |token| {
            Diagnostic::error("`)` expected after `include path")
                .with_label(Label::primary(&token, "`)` expected here"))
        }) else {
            return;
        };
    }

    pub fn preprocess(&mut self) {
        loop {
            let token = self.tokens.next();
            match token.kind {
                TokenKind::Accent => {
                    self.flush();
                    self.parse_macro_invocation(token);
                }
                TokenKind::EndOfFile => {
                    self.current_span = self.current_span.join(&TokenSpan::single(token.id));
                    break;
                }
                _ => self.current_span = self.current_span.join(&TokenSpan::single(token.id)),
            }
        }
        self.flush();
    }
}
