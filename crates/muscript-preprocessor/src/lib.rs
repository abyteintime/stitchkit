use std::{cmp::Ordering, collections::HashMap};

use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
};
use muscript_lexer::{
    sliced_tokens::{SlicedTokens, TokenSlice},
    sources::LexedSources,
    token::{AnyToken, Token, TokenKind, TokenSpan},
    token_stream::{TokenSpanCursor, TokenStream},
};

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
pub struct Preprocessor<'a> {
    global_definitions: &'a mut Definitions,
    local_definitions: Definitions,
    sources: LexedSources<'a>,
    tokens: TokenSpanCursor<'a>,
    diagnostics: &'a mut dyn DiagnosticSink<Token>,
    out_tokens: &'a mut SlicedTokens,
    current_span: TokenSpan,
    if_stack: Vec<If>,
}

#[derive(Debug, Clone, Copy)]
struct If {
    condition: bool,
    if_ident: AnyToken,
}

impl<'a> Preprocessor<'a> {
    pub fn new(
        definitions: &'a mut Definitions,
        sources: LexedSources<'a>,
        in_tokens: TokenSpanCursor<'a>,
        out_tokens: &'a mut SlicedTokens,
        diagnostics: &'a mut dyn DiagnosticSink<Token>,
    ) -> Self {
        Self {
            global_definitions: definitions,
            local_definitions: Definitions::default(),
            sources,
            tokens: in_tokens,
            diagnostics,
            out_tokens,
            current_span: TokenSpan::Empty,
            if_stack: vec![],
        }
    }

    fn flush(&mut self) {
        if let TokenSpan::Spanning { start, end } = self.current_span {
            self.out_tokens.push_slice(TokenSlice::Span { start, end });
            self.current_span = TokenSpan::Empty;
        }
    }
}

/// # Parsing primitives
impl<'a> Preprocessor<'a> {
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
impl<'a> Preprocessor<'a> {
    fn parse_macro_invocation(&mut self, accent: AnyToken) {
        let Some(macro_name_ident) = self.parse_macro_name() else {
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
            _ if macro_name.eq_ignore_ascii_case("if") => self.parse_if(macro_name_ident),
            _ if macro_name.eq_ignore_ascii_case("else") => self.parse_else(macro_name_ident),
            _ if macro_name.eq_ignore_ascii_case("endif") => self.parse_endif(macro_name_ident),
            _ if macro_name.eq_ignore_ascii_case("include") => self.parse_include(macro_name_ident),
            _ => self.parse_user_macro(macro_name_ident),
        }
    }

    fn parse_macro_name(&mut self) -> Option<AnyToken> {
        fn macro_name_expected(token: AnyToken) -> Diagnostic<Token> {
            Diagnostic::error("macro name expected")
                .with_label(Label::primary(&token, "identifier expected here"))
        }

        let name_or_left_brace = self.tokens.next();
        match name_or_left_brace.kind {
            TokenKind::Ident => Some(name_or_left_brace),
            TokenKind::LeftBrace => {
                let Some(name) = self.expect_token(TokenKind::Ident, macro_name_expected) else {
                    return None;
                };
                let Some(_right_brace) = self.expect_token(TokenKind::RightBrace, |token| {
                    Diagnostic::error("`}` expected after the braced macro name")
                        .with_label(Label::primary(&token, "`}` expected here"))
                }) else {
                    return None;
                };
                Some(name)
            }
            _ => {
                self.diagnostics
                    .emit(macro_name_expected(name_or_left_brace));
                None
            }
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
        if let Some(_old) = self.global_definitions.map.insert(
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
            .global_definitions
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
            .global_definitions
            .map
            .contains_key(CaseInsensitive::new_ref(macro_name));
        let emit_non_empty = if not { !is_defined } else { is_defined };

        if emit_non_empty {
            self.out_tokens.push_slice(TokenSlice::Span {
                start: accent.id,
                end: right_paren.id,
            });
        } else {
            self.out_tokens
                .push_slice(TokenSlice::Empty { source: accent.id });
        }
    }

    fn parse_if(&mut self, if_ident: AnyToken) {
        let Some(left_paren) = self.expect_token(TokenKind::LeftParen, |token| {
            Diagnostic::error("`(` expected after `if")
                .with_label(Label::primary(&token, "`(` expected here"))
        }) else {
            return;
        };

        // Find the condition. Initially we want to find out which tokens we should run through
        // a sub-preprocessor; for that we need to handle nesting, as `if(`isdefined(MACRO)) has
        // nested parentheses and we do not want to stop parsing at the first `)`.
        let condition_start = self.tokens.position();
        let mut token_count = 0;
        let mut nesting: u32 = 1;
        loop {
            let token = self.tokens.next();
            token_count += 1;
            match token.kind {
                TokenKind::LeftParen => nesting += 1,
                TokenKind::RightParen => {
                    nesting -= 1;
                    if nesting == 0 {
                        token_count -= 1;
                        break;
                    }
                }
                TokenKind::EndOfFile => {
                    self.diagnostics.emit(
                        Diagnostic::error("missing `)` to close `if condition").with_label(
                            Label::primary(&left_paren, "this `(` does not have a matching `)`"),
                        ),
                    );
                    return;
                }
                _ => (),
            }
        }
        let past_condition = self.tokens.position();
        self.tokens.set_position(condition_start);

        // Preprocess the condition to expand any macros inside of it and find out whether
        // the resulting token stream is empty.
        let is_condition_empty = {
            if let Some(cursor) = TokenSpanCursor::new(
                self.sources.token_arena,
                TokenSpan::spanning_len(condition_start, token_count),
            ) {
                let condition_tokens = {
                    let mut condition_tokens = SlicedTokens::new();
                    let mut sub_preprocessor = Preprocessor::new(
                        self.global_definitions,
                        self.sources,
                        cursor,
                        &mut condition_tokens,
                        self.diagnostics,
                    );
                    sub_preprocessor.preprocess();
                    condition_tokens
                };
                // We only need to check one token; if we have one, the part inside parentheses is
                // not empty.
                let condition_token = condition_tokens
                    .stream(self.sources.token_arena)
                    .map(|mut stream| stream.next());
                matches!(
                    condition_token,
                    // None can happen if the preprocessor didn't produce any tokens (which would be
                    // weird, but let's err on the side of caution.)
                    None | Some(AnyToken {
                        // EndOfFile can happen only in case of `if();
                        // FailedExp happens if any macro inside expands to nothing.
                        kind: TokenKind::EndOfFile | TokenKind::FailedExp,
                        ..
                    })
                )
            } else {
                true
            }
        };

        self.tokens.set_position(past_condition);

        // Push the `if onto the stack, so that `else, `endif, and EndOfFile know what to do.
        self.if_stack.push(If {
            condition: !is_condition_empty,
            if_ident,
        });

        if is_condition_empty {
            let Ok(()) = self.skip_until_macro(
                |name| name.eq_ignore_ascii_case("else") || name.eq_ignore_ascii_case("endif"),
                || {
                    Diagnostic::error("missing `else or `endif to close `if")
                        .with_label(Label::primary(&if_ident, "this `if is never closed"))
                },
            ) else {
                return;
            };
        }
    }

    fn parse_else(&mut self, else_ident: AnyToken) {
        if let Some(last_if) = self.if_stack.last() {
            let if_ident = last_if.if_ident;
            if last_if.condition {
                let Ok(()) = self.skip_until_macro(
                    |name| name.eq_ignore_ascii_case("endif"),
                    || {
                        Diagnostic::error("missing `endif to close `else")
                            .with_label(Label::primary(&else_ident, "this `else is never closed"))
                            .with_label(Label::primary(&if_ident, "this is the `else's `if"))
                    },
                ) else {
                    return;
                };
            }
        } else {
            self.diagnostics.emit(
                Diagnostic::error("`else without a matching `if")
                    .with_label(Label::primary(&else_ident, "stray `else here")),
            );
        }
    }

    fn skip_until_macro(
        &mut self,
        cond: impl Fn(&str) -> bool,
        eof_error: impl FnOnce() -> Diagnostic<Token>,
    ) -> Result<(), EndOfFile> {
        let mut nesting: u32 = 0;
        loop {
            let at_token = self.tokens.position();
            let token = self.tokens.next();
            match token.kind {
                TokenKind::Accent => {
                    let Some(macro_name_ident) = self.parse_macro_name() else {
                        continue;
                    };
                    let macro_name = self.sources.source(&macro_name_ident);
                    if macro_name.eq_ignore_ascii_case("if") {
                        nesting += 1;
                    } else if nesting > 0 && macro_name.eq_ignore_ascii_case("endif") {
                        nesting -= 1;
                    } else if nesting == 0 && cond(macro_name) {
                        self.tokens.set_position(at_token);
                        break;
                    }
                }
                TokenKind::EndOfFile => {
                    self.diagnostics.emit(eof_error());
                    return Err(EndOfFile);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn parse_endif(&mut self, endif_ident: AnyToken) {
        if self.if_stack.pop().is_none() {
            self.diagnostics.emit(
                Diagnostic::error("`endif without a matching `if")
                    .with_label(Label::primary(&endif_ident, "stray `endif here")),
            )
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

    fn definition(&self, name: &str) -> Option<&Definition> {
        self.local_definitions
            .map
            .get(CaseInsensitive::new_ref(name))
            .or_else(|| {
                self.global_definitions
                    .map
                    .get(CaseInsensitive::new_ref(name))
            })
    }

    fn parse_user_macro(&mut self, macro_name_ident: AnyToken) {
        let mut arguments = if self.tokens.peek().kind == TokenKind::LeftParen {
            let open = self.tokens.next();
            let (arguments, _close) = self.parse_comma_separated(open, |preprocessor| {
                let mut span = TokenSpan::Empty;
                let mut nesting = 0;
                loop {
                    let token = preprocessor.tokens.peek();
                    match token.kind {
                        TokenKind::LeftParen => {
                            _ = preprocessor.tokens.next();
                            nesting += 1;
                        }
                        TokenKind::RightParen => {
                            if nesting > 0 {
                                _ = preprocessor.tokens.next();
                                nesting -= 1;
                            } else {
                                break;
                            }
                        }
                        TokenKind::Comma if nesting == 0 => break,
                        TokenKind::EndOfFile => break,
                        _ => {
                            let token = preprocessor.tokens.next();
                            span = span.join(&TokenSpan::single(token.id));
                        }
                    }
                }
                Definition {
                    source_span: span,
                    parameters: None,
                }
            });
            Some(arguments)
        } else {
            None
        };

        let macro_name = self.sources.source(&macro_name_ident);
        if let Some(definition) = self.definition(macro_name) {
            if let Some(tokens) =
                TokenSpanCursor::new(self.sources.token_arena, definition.source_span)
            {
                let argument_count = arguments.as_ref().map(|list| list.len());
                let parameter_count = definition.parameters.as_ref().map(|list| list.len());
                match (parameter_count, argument_count) {
                    (None, Some(got)) => {
                        self.diagnostics.emit(
                            Diagnostic::error(format!(
                                "macro expected no arguments, but {got} were provided"
                            ))
                            .with_label(Label::primary(&macro_name_ident, "")),
                        );
                        self.out_tokens.push_slice(TokenSlice::Empty {
                            source: macro_name_ident.id,
                        });
                        return;
                    }
                    (Some(expected), None) => {
                        self.diagnostics.emit(
                            Diagnostic::error(format!(
                                "macro expected {expected} arguments, but none were provided"
                            ))
                            .with_label(Label::primary(&macro_name_ident, "")),
                        );
                        self.out_tokens.push_slice(TokenSlice::Empty {
                            source: macro_name_ident.id,
                        });
                        return;
                    }
                    _ => (),
                }

                if let (Some(arguments), Some(parameter_count), Some(argument_count)) =
                    (&mut arguments, parameter_count, argument_count)
                {
                    match argument_count.cmp(&parameter_count) {
                        Ordering::Equal => (),
                        // In case not enough arguments were provided, pad them with empty
                        // definitions.
                        Ordering::Less => arguments.resize_with(parameter_count, || Definition {
                            source_span: TokenSpan::Empty,
                            parameters: None,
                        }),
                        // In case too many arguments were provided, treat the extra ones as one
                        // big argument.
                        Ordering::Greater => {
                            let trailing = arguments.split_off(parameter_count - 1);
                            let first = trailing.first().expect("argument_count > parameter_count");
                            let last = trailing.last().expect("argument_count > parameter_count");
                            arguments.push(Definition {
                                source_span: first.source_span.join(&last.source_span),
                                parameters: None,
                            })
                        }
                    }
                }

                let mut sub_preprocessor = Preprocessor::new(
                    self.global_definitions,
                    self.sources,
                    tokens,
                    self.out_tokens,
                    self.diagnostics,
                );

                sub_preprocessor.preprocess();
            } else {
                // The macro is defined as empty; this is fine.
                // This case does not result in a failed expansion.
            }
        } else {
            self.out_tokens.push_slice(TokenSlice::Empty {
                source: macro_name_ident.id,
            });
        }
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

#[derive(Debug, Clone, Copy)]
struct EndOfFile;
