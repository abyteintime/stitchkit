use std::{collections::HashMap, rc::Rc};

use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{SourceFileId, Span},
};
use tracing::{trace, trace_span};
use unicase::UniCase;

use super::{
    token::{Token, TokenKind},
    LexError, Lexer, TokenStream,
};

/// A map of definitions. These may be constructed externally, to provide the preprocessor with
/// symbols such as FINAL_RELEASE.
#[derive(Debug, Clone)]
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
    pub params: Vec<String>,
}

/// Preprocessor that sits between the lexer and the parser.
///
/// Note that the MuScript preprocessor does not strive for 100% compatibility with the UnrealScript
/// preprocessor, largely because that would require sacrificing a lot of MuScript's error reporting
/// infrastructure. Therefore this preprocessor only really supports features on a best effort
/// basis; only enough features are supported to compile the engine successfully.
pub struct Preprocessor<'a> {
    pub definitions: &'a mut Definitions,
    stack: Vec<Expansion>,
}

struct Expansion {
    invocation_site: Option<(SourceFileId, Span)>,
    lexer: Lexer,
    if_stack: Vec<If>,
}

struct If {
    condition: bool,
    open: Span,
}

impl<'a> Preprocessor<'a> {
    pub fn new(file: SourceFileId, input: Rc<str>, definitions: &'a mut Definitions) -> Self {
        Self {
            definitions,
            stack: vec![Expansion {
                invocation_site: None,
                lexer: Lexer::new(file, input),
                if_stack: vec![],
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

    fn parse_define(&mut self) -> Result<(), LexError> {
        let macro_name = self.expect_token(TokenKind::Ident, |file, token| {
            Diagnostic::error(file, "new macro name expected")
                .with_label(Label::primary(token.span, "identifier expected here"))
        })?;

        if self.lexer_mut().peek()?.kind == TokenKind::LeftParen {
            todo!("macro parameters")
        }

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
                params: vec![],
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
        let macro_name = self.expect_token(TokenKind::LeftParen, |file, token| {
            Diagnostic::error(file, "missing name of macro to undefine")
                .with_label(Label::primary(token.span, "identifier expected here"))
                .with_note(NOTE)
        })?;
        let _right_paren = self.expect_token(TokenKind::LeftParen, |file, token| {
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
        let start = self.lexer().position;
        let mut nesting = 1;
        let end = loop {
            let before_token = self.lexer().position;
            let token = self.lexer_mut().next_include_comments()?;
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
                        break before_token;
                    }
                }
                _ => (),
            }
        };

        // We wanna check if the thing we expanded to is non-empty, so start should be
        // non-equal to end.
        let condition = start != end;
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

        let mut nesting = 0;
        loop {
            let before_token = self.lexer().position;
            let token = self.lexer_mut().next_include_comments()?;
            match token.kind {
                TokenKind::Accent => {
                    let macro_name_span = self.parse_macro_name()?;
                    let macro_name = macro_name_span.get_input(&self.lexer().input);
                    let macro_name = UniCase::new(macro_name);
                    // We need to keep track of nesting levels to skip over nested `ifs.
                    if macro_name == UniCase::ascii("if") {
                        trace!("if");
                        nesting += 1;
                    } else if nesting > 0 && macro_name == UniCase::ascii("endif") {
                        trace!("endif");
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

    fn parse_include(&mut self) {
        // TODO: Emit warning that MuScript does not process `includes.
    }

    fn parse_isdefined(&mut self, not: bool) -> Result<Token, LexError> {
        todo!("preprocessor `isdefined")
    }

    fn parse_user_macro(&mut self, invocation_span: Span) -> Result<(), LexError> {
        // Kind of a shame we have to allocate a whole String here, but eh. Whatever.
        let macro_name = UniCase::new(String::from(invocation_span.get_input(&self.lexer().input)));
        let file = self.lexer().file;
        if let Some(definition) = self.definitions.map.get(&macro_name) {
            trace!(?macro_name, "entering expansion");
            self.stack.push(Expansion {
                invocation_site: Some((file, invocation_span)),
                lexer: Lexer {
                    position: definition.span.start,
                    ..Lexer::new(definition.source_file, Rc::clone(&definition.text))
                },
                if_stack: vec![],
            })
        } else {
            // TODO: Emit a warning or error? Needs verification of what UPP does
        }
        Ok(())
    }

    fn parse_invocation(&mut self) -> Result<Option<Token>, LexError> {
        let macro_name_span = self.parse_macro_name()?;
        let macro_name = UniCase::new(macro_name_span.get_input(&self.lexer().input));
        trace!("Invoking macro `{macro_name}");

        match () {
            _ if macro_name == UniCase::ascii("define") => self.parse_define()?,
            _ if macro_name == UniCase::ascii("undefine") => self.parse_undefine()?,
            _ if macro_name == UniCase::ascii("if") => self.parse_if(macro_name_span)?,
            _ if macro_name == UniCase::ascii("else") => self.parse_else(macro_name_span)?,
            _ if macro_name == UniCase::ascii("endif") => self.parse_endif(macro_name_span)?,
            _ if macro_name == UniCase::ascii("include") => self.parse_include(),
            _ if macro_name == UniCase::ascii("isdefined") => {
                return Ok(Some(self.parse_isdefined(false)?))
            }
            _ if macro_name == UniCase::ascii("notdefined") => {
                return Ok(Some(self.parse_isdefined(true)?))
            }
            _ => self.parse_user_macro(macro_name_span)?,
        }

        Ok(None)
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
            TokenKind::Accent => {
                // Parse an invocation and continue executing to either parse more invocations
                // inside this one, or actually return a token to the caller.
                self.parse_invocation()?;
            }
            TokenKind::Backslash => todo!("\\ and \\n handling in preprocessor"),
            TokenKind::EndOfFile if self.is_in_expansion() => {
                trace!("exiting expansion");
                self.stack.pop();
            }
            _ => return Ok(PreprocessResult::Ignored(token)),
        }
        Ok(PreprocessResult::Consumed)
    }
}

enum PreprocessResult {
    Ignored(Token),
    Consumed,
    Produced(Token),
}

impl<'a> TokenStream for Preprocessor<'a> {
    fn next_include_comments(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.lexer_mut().next_include_comments()?;
            // eprintln!("exp:{} {:?}", self.stack.len(), token);
            match self.do_preprocess(token)? {
                PreprocessResult::Ignored(token) => return Ok(token),
                PreprocessResult::Consumed => continue,
                PreprocessResult::Produced(byproduct) => return Ok(byproduct),
            }
        }
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        // NOTE: Preprocessor inside braced strings? Is it going to be necessary?
        // We don't support exporting C++ anyways (since Hat has no way of loading our DLLs,
        // nor do we have access to the Unreal 3 C++ API) so what would be the purpose?
        self.lexer_mut().braced_string(left_brace_span)
    }

    fn peek_include_comments(&mut self) -> Result<Token, LexError> {
        let before = self.lexer().position;
        loop {
            let token = self.lexer_mut().peek_include_comments()?;
            if Self::is_preprocessor_token(token.kind) {
                let token = self.lexer_mut().next_include_comments()?;
                match self.do_preprocess(token)? {
                    PreprocessResult::Ignored(token) => {
                        // This can happen on EOF that is not significant to the preprocessor.
                        // In that case we don't need to backtrack.
                        return Ok(token);
                    }
                    PreprocessResult::Consumed => (),
                    PreprocessResult::Produced(byproduct) => {
                        self.lexer_mut().position = before;
                        return Ok(byproduct);
                    }
                };
            } else {
                return Ok(token);
            }
        }
    }

    fn peek(&mut self) -> Result<Token, LexError> {
        let before = self.lexer().position;
        loop {
            let token = self.lexer_mut().peek()?;
            if Self::is_preprocessor_token(token.kind) {
                let token = self.lexer_mut().next()?;
                match self.do_preprocess(token)? {
                    PreprocessResult::Ignored(token) => {
                        // This can happen on EOF that is not significant to the preprocessor.
                        // In that case we don't need to backtrack.
                        return Ok(token);
                    }
                    PreprocessResult::Consumed => (),
                    PreprocessResult::Produced(byproduct) => {
                        self.lexer_mut().position = before;
                        return Ok(byproduct);
                    }
                };
            } else {
                return Ok(token);
            }
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
