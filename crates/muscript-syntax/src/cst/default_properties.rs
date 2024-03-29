use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, Label},
    span::Spanned,
};
use muscript_lexer::{token::Token, token_stream::TokenStream};
use muscript_syntax_derive::Spanned;

use crate::{
    diagnostics::notes,
    list::{SeparatedListDiagnostics, TerminatedListErrorKind},
    token::{
        Add, AnyToken, Assign, Dot, FailedExp, FloatLit, Ident, IntLit, LeftBrace, LeftBracket,
        LeftParen, NameLit, RightBrace, RightBracket, RightParen, Semi, StringLit, Sub,
    },
    Parse, ParseError, Parser, PredictiveParse,
};

use super::Path;

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct DefaultPropertiesBlock {
    pub open: LeftBrace,
    pub properties: Vec<DefaultProperty>,
    pub close: RightBrace,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "default_property_error")]
pub enum DefaultProperty {
    Subobject(Subobject),
    Value(Value),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Value {
    pub key: Key,
    pub action: ValueAction,
    pub semi: Option<Semi>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Key {
    pub ident: Ident,
    pub index: Option<Index>,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "index_error")]
pub enum Index {
    Parens(LeftParen, IndexLit, RightParen),
    Brackets(LeftBracket, IndexLit, RightBracket),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "index_lit_error")]
pub enum IndexLit {
    Num(IntLit),
    Enum(Path),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "value_action_error")]
pub enum ValueAction {
    Assign(Assign, Lit),
    Call(Dot, Ident, Option<CallArg>),
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct CallArg {
    pub open: LeftParen,
    pub expr: Option<Lit>,
    pub close: RightParen,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "num_lit_error")]
pub enum NumLit {
    Int(IntLit),
    Float(FloatLit),
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "lit_error")]
pub enum Lit {
    FailedExp(FailedExp),
    Num(NumLit),
    Pos(Add, NumLit),
    Neg(Sub, NumLit),
    String(StringLit),
    Ident(Ident, Option<NameLit>),
    Compound(BracedCompound),
}

/// `Compound` with optional braces.
///
/// This was required in vanilla UnrealScript in order for the `defaultproperties` parser to ignore
/// newlines within compound literals, but MuScript does not have such limitations; this exists
/// solely for compatibility purposes.
#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "braced_compound_error")]
pub enum BracedCompound {
    Braced(LeftBrace, Compound, RightBrace),
    Bare(Compound),
}

/// Compound data structure (array or struct.)
///
/// At the parsing stage they can be mixed freely, but semantic analysis rejects listings where
/// both appear at the same time.
#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Compound {
    pub open: LeftParen,
    pub elements: Vec<CompoundElement>,
    pub close: RightParen,
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub enum CompoundElement {
    Lit(Lit),
    Field(Key, Assign, Lit),
}

keyword! {
    KBegin = "begin",
    KEnd = "end",
    KObject = "object",
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Subobject {
    pub begin: KBegin,
    pub object1: KObject,
    pub properties: Vec<Value>,
    pub end: KEnd,
    pub object2: KObject,
}

impl Parse for DefaultPropertiesBlock {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open: LeftBrace = parser.parse()?;
        let (properties, close) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error("missing `}` to close default properties block")
                        .with_label(Label::primary(&open, "this is where the block begins")),
                );
            }
            error.parse
        })?;
        Ok(Self {
            open,
            properties,
            close,
        })
    }
}

impl Parse for CallArg {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open = parser.parse()?;
        if let Some(close) = parser.parse()? {
            Ok(Self {
                open,
                expr: None,
                close,
            })
        } else {
            Ok(Self {
                open,
                expr: Some(parser.parse()?),
                close: parser.parse()?,
            })
        }
    }
}

impl Parse for Compound {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let open = parser.parse()?;
        let (elements, close) = parser.parse_comma_separated_list().map_err(|error| {
            parser.emit_separated_list_diagnostic(
                &open,
                error,
                SeparatedListDiagnostics {
                    missing_right: "missing `)` to close compound literal",
                    missing_right_label: "this `(` does not have a matching `)`",
                    missing_comma: "`,` or `)` expected",
                    missing_comma_open: "the compound literal starts here",
                    missing_comma_token: "this was expected to continue or close the literal",
                    missing_comma_note:
                        "note: elements in compound literals are separated by commas `,`",
                },
            )
        })?;
        Ok(Self {
            open,
            elements,
            close,
        })
    }
}

impl Parse for CompoundElement {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        if let Some(ident) = parser.parse()? {
            if let index @ Some(_) = parser.parse()? {
                Ok(Self::Field(
                    Key { ident, index },
                    parser.parse()?,
                    parser.parse()?,
                ))
            } else if let Some(assign) = parser.parse()? {
                Ok(Self::Field(
                    Key { ident, index: None },
                    assign,
                    parser.parse()?,
                ))
            } else {
                Ok(Self::Lit(Lit::Ident(ident, parser.parse()?)))
            }
        } else {
            Ok(Self::Lit(parser.parse()?))
        }
    }
}

impl Parse for Subobject {
    fn parse(parser: &mut Parser<'_, impl TokenStream>) -> Result<Self, ParseError> {
        let begin: KBegin = parser.parse()?;
        let object1: KObject = parser.parse()?;
        let (properties, end) = parser.parse_terminated_list().map_err(|error| {
            if let TerminatedListErrorKind::MissingTerminator = error.kind {
                parser.emit_diagnostic(
                    Diagnostic::error("missing `end object` to end default subobject").with_label(
                        Label::primary(
                            &begin.span().join(&object1.span()),
                            "this `begin object` does not have a matching `end object`",
                        ),
                    ),
                );
            }
            error.parse
        })?;
        let object2 = parser.parse()?;
        Ok(Self {
            begin,
            object1,
            properties,
            end,
            object2,
        })
    }
}

fn default_property_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("default property expected")
        .with_label(Label::primary(
            token,
            "this token does not start a default property",
        ))
        .with_note(indoc!(
            r#"note: default property declarations can take one of the following forms:
                   - `Key = Value`
                   - `begin object .. end object` (case-insensitive)
                   - `Array.Operation(OptionalArgs)`
            "#,
        ))
}

fn index_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("`(Index)` or `[Index]` expected")
        .with_label(Label::primary(token, "array index expected here"))
}

fn index_lit_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("integer or enum index expected")
        .with_label(Label::primary(token, "array index expected here"))
        .with_note("note: indices are integers `[1]`, or enums `[EXAMPLE_EnumValue]`")
}

fn value_action_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("`=` or `.Operation(Arg)` expected")
        .with_label(Label::primary(token, "property action expected here"))
}

fn num_lit_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("number literal expected")
        .with_label(Label::primary(token, "number literal expected here"))
}

fn lit_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("default property literal expected")
        .with_label(Label::primary(
            token,
            "this token is not a supported literal",
        ))
        .with_note("note: values of default properties must be literal and cannot be expressions")
        .with_note(indoc!(
            r#"note: the syntax of certain literals is different from normal UnrealScript
                   - name literals are not enclosed in apostrophes;
                     for `var name Example;` its default value can be specified using `Example = Something`
                   - unlike in normal code, arrays and structs have literals
                     arrays:  `(1, 2, 3)`
                     structs: `(X=1, Y=2, Z=3)` - instead of `vect(1, 2, 3)`
            "#
        ))
}

fn braced_compound_error(_: &Parser<'_, impl TokenStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::bug("compound literal expected")
        .with_label(Label::primary(token, "compound literal expected here"))
        .with_note(notes::PARSER_BUG)
        .with_note("help: do not try to parse BracedCompound directly; Lit should be used instead")
}
