use std::{collections::HashMap, rc::Rc};

use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{SourceFileId, Span},
};
use tracing::trace;
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
    macro_name: Option<String>,
    lexer: Lexer,
}

impl<'a> Preprocessor<'a> {
    pub fn new(file: SourceFileId, input: Rc<str>, definitions: &'a mut Definitions) -> Self {
        Self {
            definitions,
            stack: vec![Expansion {
                macro_name: None,
                lexer: Lexer::new(file, input),
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
        todo!("preprocessor `undefine")
    }

    fn parse_if(&mut self) -> Result<(), LexError> {
        todo!("preprocessor `if")
    }

    fn parse_include(&mut self) -> Result<(), LexError> {
        // NOTE: How would `include even work?
        todo!("preprocessor `include")
    }

    fn parse_isdefined(&mut self) -> Result<(), LexError> {
        todo!("preprocessor `isdefined")
    }

    fn parse_user_macro(&mut self, macro_name: &str) -> Result<(), LexError> {
        if let Some(definition) = self
            .definitions
            .map
            // Kind of a shame we have to allocate a whole String here, but eh. Whatever.
            .get(&UniCase::new(String::from(macro_name)))
        {
            self.stack.push(Expansion {
                macro_name: Some(String::from(macro_name)),
                lexer: Lexer {
                    position: definition.span.start,
                    ..Lexer::new(definition.source_file, Rc::clone(&definition.text))
                },
            })
        } else {
            // TODO: Emit a warning or error? Needs verification of what UPP does
        }
        Ok(())
    }

    fn parse_invocation(&mut self) -> Result<(), LexError> {
        let macro_name_span = self.parse_macro_name()?;
        let input = Rc::clone(&self.lexer().input);
        let macro_name = macro_name_span.get_input(&input);

        match macro_name {
            "define" => self.parse_define()?,
            "undefine" => self.parse_undefine()?,
            "if" => self.parse_if()?,
            "include" => self.parse_include()?,
            "isdefined" => self.parse_isdefined()?,
            _ => self.parse_user_macro(macro_name)?,
        }

        Ok(())
    }

    fn is_preprocessor_token(kind: TokenKind) -> bool {
        matches!(kind, TokenKind::Accent | TokenKind::Backslash)
    }

    fn do_preprocess(&mut self, token: Token) -> Result<Option<Token>, LexError> {
        match token.kind {
            TokenKind::Accent => {
                // Parse an invocation and continue executing to either parse more invocations
                // inside this one, or actually return a token to the caller.
                self.parse_invocation()?;
            }
            TokenKind::Backslash => todo!("\\ and \\n handling in preprocessor"),
            TokenKind::EndOfFile if self.is_in_expansion() => {
                self.stack.pop();
            }
            _ => return Ok(Some(token)),
        }
        Ok(None)
    }
}

impl<'a> TokenStream for Preprocessor<'a> {
    fn next_include_comments(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.lexer_mut().next_include_comments()?;
            if let Some(token) = self.do_preprocess(token)? {
                return Ok(token);
            }
        }
    }

    fn braced_string(&mut self, left_brace_span: Span) -> Result<Span, LexError> {
        // NOTE: Preprocessor inside braced strings? Is it going to be necessary?
        // Braced strings are exclusively used for cpptext, and there the C++ preprocessor is
        // available and used instead. So is there any incentive for us to support it there?
        // (Note that unlike UnrealScript, MuScript parses default properties as part of its
        //  own syntax; they are not braced strings.)
        self.lexer_mut().braced_string(left_brace_span)
    }

    fn peek_include_comments(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.lexer_mut().peek_include_comments()?;
            if Self::is_preprocessor_token(token.kind) {
                let token = self.lexer_mut().next_include_comments()?;
                self.do_preprocess(token)?;
            } else {
                return Ok(token);
            }
        }
    }

    fn peek(&mut self) -> Result<Token, LexError> {
        loop {
            let token = self.lexer_mut().peek()?;
            if Self::is_preprocessor_token(token.kind) {
                let token = self.lexer_mut().next()?;
                self.do_preprocess(token)?;
            } else {
                return Ok(token);
            }
        }
    }
}
