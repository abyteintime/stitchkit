use std::{cmp::Ordering, collections::HashMap, rc::Rc};

use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{SourceFileId, Span},
};
use tracing::{trace, trace_span};
use unicase::UniCase;

use crate::diagnostics::DiagnosticSink;

use super::{
    token::{Token, TokenKind},
    Channel, EofReached, LexError, Lexer, TokenStream,
};

/// A map of definitions. These may be constructed externally, to provide the preprocessor with
/// symbols such as `FINAL_RELEASE`.
#[derive(Debug, Clone, Default)]
pub struct Definitions {
    pub map: HashMap<UniCase<String>, Definition>,
}

/// A single preprocessor definition.
#[derive(Debug, Clone)]
pub struct Definition {
    /// The source file where the symbol is defined.
    pub source_file: SourceFileId,
    /// The span the symbol occupies within the source file.
    pub span: Span,
    /// The definition text.
    pub text: Rc<str>,
    /// The parameters of this definition.
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
    pub definitions: &'a mut Definitions,
    errors: &'a mut dyn DiagnosticSink,
    stack: Vec<Expansion>,
}

#[derive(Debug, Clone)]
struct Expansion {
    invocation_site: Option<(SourceFileId, Span)>,
    lexer: Lexer,
    if_stack: Vec<If>,
    arguments: Definitions,
}

#[derive(Debug, Clone)]
struct If {
    condition: bool,
    open: Span,
}

impl<'a> Preprocessor<'a> {
    pub fn new(
        file: SourceFileId,
        input: Rc<str>,
        definitions: &'a mut Definitions,
        errors: &'a mut dyn DiagnosticSink,
    ) -> Self {
        Self {
            definitions,
            errors,
            stack: vec![Expansion {
                invocation_site: None,
                lexer: Lexer::new(file, input),
                if_stack: vec![],
                arguments: Default::default(),
            }],
        }
    }

    fn current_expansion(&self) -> &Expansion {
        self.stack
            .last()
            .expect("preprocessor expansion stack must not be empty")
    }

    fn current_expansion_mut(&mut self) -> &mut Expansion {
        self.stack
            .last_mut()
            .expect("preprocessor expansion stack must not be empty")
    }

    fn is_in_expansion(&self) -> bool {
        self.stack.len() > 1
    }

    fn lexer_mut(&mut self) -> &mut Lexer {
        &mut self.current_expansion_mut().lexer
    }

    fn lexer(&self) -> &Lexer {
        &self.current_expansion().lexer
    }

    // We don't have the expansive parsing machinery of Parser<T> here, so we have to do a little
    // bit of manual parsing here.
    fn expect_token(
        &mut self,
        kind: TokenKind,
        error: impl FnOnce(SourceFileId, Token) -> Diagnostic,
    ) -> Result<Token, LexError> {
        let token = self.lexer_mut().next()?;
        if token.kind != kind {
            return Err(LexError::new(
                token.span,
                error(self.lexer_mut().file, token),
            ));
        }
        Ok(token)
    }

    fn parse_macro_name(&mut self) -> Result<Span, LexError> {
        let token = self.lexer_mut().next()?;
        Ok(match token.kind {
            TokenKind::Ident => token.span,
            TokenKind::LeftBrace => {
                let name = self.expect_token(TokenKind::Ident, |file, token| {
                    Diagnostic::error(file, "macro name expected inside `{}`")
                        .with_label(Label::primary(token.span, "macro name expected here"))
                        .with_note("note: ``{}` invokes the preprocessor")
                })?;
                let _right_brace = self.expect_token(TokenKind::RightBrace, |file, token| {
                    Diagnostic::error(file, "`}` expected after braced macro name")
                        .with_label(Label::primary(token.span, "`}` expected here"))
                        .with_note("note: ``{}` invokes the preprocessor")
                })?;
                name.span
            }
            _ => {
                return Err(LexError::new(
                    token.span,
                    Diagnostic::error(
                        self.lexer_mut().file,
                        "macro name or `{` expected after ```",
                    )
                    .with_label(Label::primary(
                        token.span,
                        "identifier or `{` expected here",
                    ))
                    .with_note("note: ``` invokes the preprocessor"),
                ))
            }
        })
    }

    fn parse_comma_separated<T>(
        &mut self,
        left_paren: &Token,
        mut parse: impl FnMut(&mut Self) -> Result<T, LexError>,
    ) -> Result<(Vec<T>, Token), LexError> {
        // This is more or less just duplicating the logic in parse_separated_list.
        // It ain't nice, but coaxing the parser to do preprocessor work ain't it either.

        // Unfortunately the diagnostics here aren't as great, mainly for simplicity's sake.
        // You shouldn't be abusing the preprocessor too hard anyways. MuScript has it for
        // compatibility reasons.
        let mut elements = vec![];
        let close = loop {
            let token = self.lexer_mut().peek()?;
            match token.kind {
                TokenKind::EndOfFile => {
                    return Err(LexError::new(
                        left_paren.span,
                        Diagnostic::error(self.lexer().file, "mismatched parenthesis").with_label(
                            Label::primary(
                                left_paren.span,
                                "this `(` does not have a matching `)`",
                            ),
                        ),
                    ));
                }
                TokenKind::RightParen => {
                    break self.lexer_mut().next()?;
                }
                _ => (),
            }
            elements.push(parse(self)?);
            let next_token = self.lexer_mut().next()?;
            match next_token.kind {
                TokenKind::Comma => (),
                TokenKind::RightParen => {
                    break next_token;
                }
                _ => {
                    return Err(LexError::new(
                        next_token.span,
                        Diagnostic::error(self.lexer().file, "`,` or `)` expected")
                            .with_label(Label::primary(next_token.span, "")),
                    ));
                }
            }
        };

        Ok((elements, close))
    }

    fn parse_define(&mut self) -> Result<(), LexError> {
        let macro_name = self.expect_token(TokenKind::Ident, |file, token| {
            Diagnostic::error(file, "new macro name expected")
                .with_label(Label::primary(token.span, "identifier expected here"))
        })?;

        let parameters = if self.lexer_mut().peek()?.kind == TokenKind::LeftParen {
            let open = self.lexer_mut().next()?;
            let (parameters, _) = self.parse_comma_separated(&open, |preproc| {
                let parameter = preproc.expect_token(TokenKind::Ident, |file, token| {
                    Diagnostic::error(file, "macro argument name expected")
                        .with_label(Label::primary(token.span, "identifier expected here"))
                })?;
                let name = parameter.span.get_input(&preproc.lexer().input);
                Ok(name.to_owned())
            })?;
            Some(parameters)
        } else {
            None
        };

        let start = self.lexer_mut().position;
        loop {
            self.lexer_mut().eat_until_line_feed();
            let here = self.lexer_mut().position;
            let text = &self.lexer_mut().input[start..here].trim_end();
            // Macros can be carried over to the next line using \.
            if text.ends_with('\\') {
                continue;
            } else {
                break;
            }
        }
        let end = self.lexer_mut().position;

        // NOTE: We include everything from the start, so that line info remains correct when lexing
        // within the file. Then when expanding we "teleport" the lexer to the correct starting
        // position.
        let source_file = self.lexer_mut().file;
        let macro_name = macro_name.span.get_input(&self.lexer().input);
        let text: Rc<str> = Rc::from(&self.lexer().input[..end]);

        trace!(
            ?source_file,
            macro_name,
            text = &text[start..],
            "Defined macro"
        );

        if let Some(_old) = self.definitions.map.insert(
            UniCase::from(String::from(macro_name)),
            Definition {
                source_file,
                span: Span::from(start..end),
                text,
                parameters,
            },
        ) {
            // TODO: Warning on redefinition of macro?
        }

        Ok(())
    }

    fn parse_undefine(&mut self) -> Result<(), LexError> {
        const NOTE: &str = indoc!(
            "note: `undefine expects a parameter containing the macro name, like:
                       `undefine(EXAMPLE_MACRO)"
        );
        let _left_paren = self.expect_token(TokenKind::LeftParen, |file, token| {
            Diagnostic::error(file, "`(` expected")
                .with_label(Label::primary(token.span, "`(` expected here"))
                .with_note(NOTE)
        })?;
        let macro_name = self.expect_token(TokenKind::Ident, |file, token| {
            Diagnostic::error(file, "missing name of macro to undefine")
                .with_label(Label::primary(token.span, "identifier expected here"))
                .with_note(NOTE)
        })?;
        let _right_paren = self.expect_token(TokenKind::RightParen, |file, token| {
            Diagnostic::error(file, "`)` expected to close `undefine invocation")
                .with_label(Label::primary(token.span, "`)` expected here"))
                .with_note(NOTE)
        })?;

        let macro_name = macro_name.span.get_input(&self.lexer().input);
        let macro_name = UniCase::new(String::from(macro_name));
        if self.definitions.map.remove(&macro_name).is_none() {
            // TODO: Warning when a macro that is never defined is undefined?
        }

        Ok(())
    }

    fn parse_if(&mut self, if_span: Span) -> Result<(), LexError> {
        let left_paren = self.expect_token(TokenKind::LeftParen, |file, token| {
            Diagnostic::error(file, "`(` expected")
                .with_label(Label::primary(token.span, "`(` expected here"))
        })?;
        // NOTE: The condition mechanism is different from vanilla UnrealScript since our
        // preprocessor operates on a higher level than plain text.
        // The most important part is that pasting arbitrary user-defined macros, as well as
        // `isdefined/`notdefined, works.
        let mut consumed_tokens: usize = 0;
        let mut nesting: usize = 1;
        loop {
            let token = self.next_any()?;
            match token.kind {
                TokenKind::EndOfFile => {
                    return Err(LexError::new(
                        left_paren.span,
                        Diagnostic::error(self.lexer().file, "missing `)` to close `if condition")
                            .with_label(Label::primary(
                                left_paren.span,
                                "the condition starts here",
                            )),
                    ))
                }
                TokenKind::LeftParen => nesting += 1,
                TokenKind::RightParen => {
                    nesting -= 1;
                    if nesting == 0 {
                        break;
                    }
                }
                _ => consumed_tokens += 1,
            }
        }

        // We wanna check if the thing we expanded to is non-empty, so start should be
        // non-equal to end.
        let condition = consumed_tokens > 0;
        // Keep track of this `if/`else, so that we can report an error on EOF if there is no
        // matching `endif.
        self.current_expansion_mut().if_stack.push(If {
            condition,
            open: if_span,
        });

        trace!(condition, "`if");

        if !condition {
            self.skip_until_macro(
                |name| name == UniCase::ascii("else") || name == UniCase::ascii("endif"),
                |file| {
                    LexError::new(
                        if_span,
                        Diagnostic::error(file, "missing `else or `endif to close `if")
                            .with_label(Label::primary(if_span, "this `if is never closed")),
                    )
                },
            )?;
        }

        Ok(())
    }

    fn parse_else(&mut self, else_span: Span) -> Result<(), LexError> {
        trace!("`else");
        if let Some(last_if) = self.current_expansion().if_stack.last() {
            let if_span = last_if.open;
            if last_if.condition {
                self.skip_until_macro(
                    |name| name == UniCase::ascii("endif"),
                    |file| {
                        LexError::new(
                            else_span,
                            Diagnostic::error(file, "missing `endif to close `else")
                                .with_label(Label::primary(else_span, "this `else is never closed"))
                                .with_label(Label::secondary(if_span, "this is the `else's `if")),
                        )
                    },
                )?;
            }
        } else {
            return Err(LexError::new(
                else_span,
                Diagnostic::error(self.lexer().file, "`else without a matching `if")
                    .with_label(Label::primary(else_span, "stray `else here")),
            ));
        }

        Ok(())
    }

    fn parse_endif(&mut self, endif_span: Span) -> Result<(), LexError> {
        trace!("`endif");
        if self.current_expansion_mut().if_stack.pop().is_none() {
            return Err(LexError::new(
                endif_span,
                Diagnostic::error(self.lexer().file, "`endif without a matching `if")
                    .with_label(Label::primary(endif_span, "stray `endif here")),
            ));
        }

        Ok(())
    }

    fn skip_until_macro(
        &mut self,
        cond: impl Fn(UniCase<&str>) -> bool,
        error: impl FnOnce(SourceFileId) -> LexError,
    ) -> Result<(), LexError> {
        let _span = trace_span!("skip_until_macro").entered();

        let mut nesting: usize = 0;
        loop {
            let before_token = self.lexer().position;
            let token = self.lexer_mut().next_any()?;
            match token.kind {
                TokenKind::Accent => {
                    let macro_name_span = self.parse_macro_name()?;
                    let macro_name = macro_name_span.get_input(&self.lexer().input);
                    let macro_name = UniCase::new(macro_name);
                    // We need to keep track of nesting levels to skip over nested `ifs.
                    if macro_name == UniCase::ascii("if") {
                        trace!("Nesting: `if");
                        nesting += 1;
                    } else if nesting > 0 && macro_name == UniCase::ascii("endif") {
                        trace!("Nesting: `endif");
                        nesting -= 1;
                    } else if nesting == 0 && cond(macro_name) {
                        trace!("Exitting");
                        self.lexer_mut().position = before_token;
                        break;
                    }
                }
                TokenKind::EndOfFile => {
                    trace!("End of file");
                    return Err(error(self.lexer().file));
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn parse_include(&mut self, span: Span) -> Result<(), LexError> {
        self.errors.emit(
            Diagnostic::warning(self.lexer().file, "use of `include preprocessor directive")
                .with_label(Label::primary(span, ""))
                .with_note("note: MuScript ignores `include directives because it processes files in the correct order automatically"),
        );

        let left_paren = self.expect_token(TokenKind::LeftParen, |file, token| {
            Diagnostic::error(file, "`(` expected")
                .with_label(Label::primary(token.span, "`(` expected here"))
        })?;

        loop {
            let token = self.lexer_mut().peek()?;
            match token.kind {
                TokenKind::RightParen => break,
                TokenKind::EndOfFile => {
                    return Err(LexError::new(
                        left_paren.span,
                        Diagnostic::error(self.lexer().file, "missing `)` to close `include path")
                            .with_label(Label::primary(
                                left_paren.span,
                                "this `(` does not have a matching `)`",
                            )),
                    ));
                }
                _ => {
                    self.lexer_mut().next()?;
                }
            }
        }

        let _right_paren = self.expect_token(TokenKind::RightParen, |file, token| {
            Diagnostic::error(file, "`)` expected after `include path")
                .with_label(Label::primary(token.span, "`)` expected here"))
        })?;

        Ok(())
    }

    fn parse_isdefined(&mut self, span: Span, not: bool) -> Result<Option<Token>, LexError> {
        const NOTE: [&str; 2] = [
            indoc!(
                "note: `isdefined expects a parameter containing the macro name, like:
                       `isdefined(EXAMPLE_MACRO)"
            ),
            indoc!(
                "note: `notdefined expects a parameter containing the macro name, like:
                       `notdefined(EXAMPLE_MACRO)"
            ),
        ];
        let _left_paren = self.expect_token(TokenKind::LeftParen, |file, token| {
            Diagnostic::error(file, "`(` expected")
                .with_label(Label::primary(token.span, "`(` expected here"))
                .with_note(NOTE[not as usize])
        })?;
        let macro_name = self.expect_token(TokenKind::Ident, |file, token| {
            Diagnostic::error(file, "missing name of macro to check")
                .with_label(Label::primary(token.span, "identifier expected here"))
                .with_note(NOTE[not as usize])
        })?;
        let _right_paren = self.expect_token(TokenKind::RightParen, |file, token| {
            Diagnostic::error(
                file,
                [
                    "`)` expected to close `isdefined invocation",
                    "`)` expected to close `notdefined invocation",
                ][not as usize],
            )
            .with_label(Label::primary(token.span, "`)` expected here"))
            .with_note(NOTE[not as usize])
        })?;

        let macro_name = macro_name.span.get_input(&self.lexer().input);
        let macro_name = UniCase::new(String::from(macro_name));
        let is_defined = self.definitions.map.contains_key(&macro_name);
        let result = if not { !is_defined } else { is_defined };

        // NOTE: This behavior is _very_ different from what UPP does, however game code does not
        // seem to use `isdefined/`notdefined outside the preprocessor, thus I think it's fine to
        // omit the invocation and not produce a token, so that `if can recognize it has no tokens
        // inside its condition and bail.
        Ok(result.then_some(Token {
            kind: TokenKind::Generated,
            span,
        }))
    }

    fn get_definition(&self, name: &str) -> Option<&Definition> {
        let name = UniCase::new(String::from(name));
        self.current_expansion()
            .arguments
            .map
            .get(&name)
            .or_else(|| self.definitions.map.get(&name))
    }

    fn parse_user_macro(&mut self, invocation_span: Span) -> Result<PreprocessResult, LexError> {
        let arguments = if self.lexer_mut().peek()?.kind == TokenKind::LeftParen {
            let open = self.lexer_mut().next()?;
            let (arguments, close) = self.parse_comma_separated(&open, |preproc| {
                let start = preproc.lexer().position;
                let mut nesting = 0;
                loop {
                    let token = preproc.lexer_mut().peek()?;
                    match token.kind {
                        TokenKind::LeftParen => {
                            let _ = preproc.lexer_mut().next()?;
                            nesting += 1;
                        }
                        TokenKind::RightParen if nesting > 0 => {
                            let _ = preproc.lexer_mut().next()?;
                            nesting -= 1;
                        }
                        TokenKind::RightParen if nesting == 0 => break,
                        TokenKind::Comma if nesting == 0 => break,
                        _ => {
                            let _ = preproc.lexer_mut().next()?;
                        }
                    }
                }
                let end = preproc.lexer().position;
                Ok(Definition {
                    source_file: preproc.lexer().file,
                    span: Span::from(start..end),
                    text: Rc::from(&preproc.lexer().input[0..end]),
                    parameters: None,
                })
            })?;
            Some((arguments, close))
        } else {
            None
        };

        let macro_name = invocation_span.get_input(&self.lexer().input);
        if let Some(definition) = self.get_definition(macro_name) {
            trace!(?macro_name, "entering expansion");

            let argument_count = arguments.as_ref().map(|(list, _)| list.len());
            let parameter_count = definition.parameters.as_ref().map(|list| list.len());
            'check: {
                return Err(LexError::new(
                    invocation_span,
                    Diagnostic::error(
                        self.lexer().file,
                        match (parameter_count, argument_count) {
                            (None, Some(arg)) => {
                                format!("macro expected no arguments, but {arg} were provided")
                            }
                            (Some(param), None) => {
                                format!("macro expected {param} arguments, but none were provided")
                            }
                            // NOTE: It's okay if a macro has too many or too little arguments.
                            // You will see why in just a moment.
                            _ => break 'check,
                        },
                    )
                    .with_label(Label::primary(invocation_span, "")),
                ));
            }

            let arguments = if let (Some(argument_count), Some(parameter_count)) =
                (argument_count, parameter_count)
            {
                let (mut arguments, close) = arguments.unwrap();
                match argument_count.cmp(&parameter_count) {
                    // The least interesting case; just pass the arguments as they are.
                    Ordering::Equal => (),
                    // In case not enough arguments were provided, pad them with emptiness.
                    Ordering::Less => {
                        arguments.resize_with(parameter_count, || Definition {
                            source_file: self.lexer().file,
                            // Just use the position right before the right parenthesis.
                            span: Span::from(close.span.start..close.span.start),
                            text: Rc::from(&self.lexer().input[..close.span.start]),
                            parameters: None,
                        });
                    }
                    // In case too many arguments were provided, treat the extra args as one big
                    // argument.
                    Ordering::Greater => {
                        let trailing = arguments.split_off(parameter_count - 1);
                        let first = trailing.first().unwrap();
                        let last = trailing.last().unwrap();
                        arguments.push(Definition {
                            source_file: self.lexer().file,
                            span: Span::from(first.span.start..last.span.end),
                            text: Rc::from(&self.lexer().input[..last.span.end]),
                            parameters: None,
                        });
                    }
                }
                Some(arguments)
            } else {
                None
            };

            let file = self.lexer().file;
            self.stack.push(Expansion {
                invocation_site: Some((file, invocation_span)),
                lexer: Lexer {
                    position: definition.span.start,
                    ..Lexer::new(definition.source_file, Rc::clone(&definition.text))
                },
                if_stack: vec![],
                arguments: arguments
                    .map(|values| Definitions {
                        map: values
                            .into_iter()
                            .enumerate()
                            .map(|(i, parameter_definition)| {
                                (
                                    UniCase::new(
                                        definition.parameters.as_ref().unwrap()[i].clone(),
                                    ),
                                    parameter_definition,
                                )
                            })
                            .collect(),
                    })
                    .unwrap_or_default(),
            });
            Ok(PreprocessResult::Consumed)
        } else {
            Ok(PreprocessResult::Produced(Token {
                kind: TokenKind::FailedExp,
                span: invocation_span,
            }))
        }
    }

    fn parse_invocation(&mut self) -> Result<PreprocessResult, LexError> {
        let span = self.parse_macro_name()?;
        let macro_name = UniCase::new(span.get_input(&self.lexer().input));
        trace!("Invoking macro `{macro_name}");

        match () {
            _ if macro_name == UniCase::ascii("define") => self.parse_define()?,
            _ if macro_name == UniCase::ascii("undefine") => self.parse_undefine()?,
            _ if macro_name == UniCase::ascii("if") => self.parse_if(span)?,
            _ if macro_name == UniCase::ascii("else") => self.parse_else(span)?,
            _ if macro_name == UniCase::ascii("endif") => self.parse_endif(span)?,
            _ if macro_name == UniCase::ascii("include") => self.parse_include(span)?,
            _ if macro_name == UniCase::ascii("isdefined") => {
                return Ok(self
                    .parse_isdefined(span, false)?
                    .map(PreprocessResult::Produced)
                    .unwrap_or(PreprocessResult::Consumed))
            }
            _ if macro_name == UniCase::ascii("notdefined") => {
                return Ok(self
                    .parse_isdefined(span, true)?
                    .map(PreprocessResult::Produced)
                    .unwrap_or(PreprocessResult::Consumed))
            }
            _ => return self.parse_user_macro(span),
        }

        Ok(PreprocessResult::Consumed)
    }

    fn parse_backslash(&mut self) -> Result<(), LexError> {
        // Skip \n, which MuScript parses as two tokens `\` and `n`.
        let next_token = self.lexer_mut().peek()?;
        if next_token.span.get_input(&self.lexer().input) == "n" {
            let _ = self.lexer_mut().next()?;
        }
        Ok(())
    }

    /// Returns whether the token is significant for the preprocessor.
    fn is_preprocessor_token(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Accent | TokenKind::Backslash | TokenKind::EndOfFile
        )
    }

    fn do_preprocess(&mut self, token: Token) -> Result<PreprocessResult, LexError> {
        match token.kind {
            TokenKind::Accent => self.parse_invocation(),
            TokenKind::Backslash => {
                self.parse_backslash()?;
                Ok(PreprocessResult::Consumed)
            }
            TokenKind::EndOfFile if self.is_in_expansion() => {
                trace!("exiting expansion");
                self.stack.pop();
                Ok(PreprocessResult::Consumed)
            }
            _ => Ok(PreprocessResult::Ignored(token)),
        }
    }
}

#[must_use]
enum PreprocessResult {
    Ignored(Token),
    Consumed,
    Produced(Token),
}

impl<'a> TokenStream for Preprocessor<'a> {
    fn next_any(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.lexer_mut().next_any()?;
            match self.do_preprocess(token)? {
                PreprocessResult::Ignored(token) => return Ok(token),
                PreprocessResult::Consumed => continue,
                PreprocessResult::Produced(byproduct) => return Ok(byproduct),
            }
        }
    }

    fn text_blob(&mut self, is_end: &dyn Fn(char) -> bool) -> Result<Span, EofReached> {
        self.lexer_mut().text_blob(is_end)
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        // NOTE: Preprocessor inside braced strings? Is it going to be necessary?
        // We don't support exporting C++ anyways (since Hat has no way of loading our DLLs,
        // nor do we have access to the Unreal 3 C++ API) so what would be the purpose?
        self.lexer_mut().braced_string(left_brace_span)
    }

    fn peek_from(&mut self, channel: Channel) -> Result<Token, LexError> {
        let lexed_token = self.lexer_mut().peek_from(channel)?;
        if Self::is_preprocessor_token(lexed_token.kind) {
            // This might seem a little slow given that this should be a "simple" peek
            // operation, but remember that most tokens are not relevant for the preprocessor
            // and as such use the fast path (this `if`'s `else` branch.)
            let mut sub = Preprocessor {
                definitions: self.definitions,
                // Peek must not emit any diagnostics.
                errors: &mut (),
                stack: self.stack.clone(),
            };
            let preprocessed_token = sub.next_from(channel)?;
            Ok(preprocessed_token)
        } else {
            Ok(lexed_token)
        }
    }

    fn contextualize_diagnostic(&self, mut diagnostic: Diagnostic) -> Diagnostic {
        for expansion in self.stack[1..].iter().rev() {
            if let Some((file, span)) = expansion.invocation_site {
                diagnostic = diagnostic.with_child(
                    Diagnostic::note(file, "this error occurred while expanding a macro")
                        .with_label(Label::primary(span, "the error occurred inside this macro")),
                );
            }
        }
        diagnostic
    }
}
