use muscript_foundation::errors::{Diagnostic, Label};
use muscript_syntax_derive::Spanned;

use crate::{
    diagnostics::{labels, notes},
    lexis::token::{AnyToken, Ident, LeftParen, RightParen, Semi, Token},
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{BoolLit, Path, SpecifierArgs};

keyword! {
    KPartial = "partial",
    KClass = "class",
    KInterface = "interface",
    KExtends = "extends",
    KWithin = "within",
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
#[parse(error = "class_kind_error")]
pub enum ClassKind {
    Class(KClass),
    PartialClass(KPartial, KClass),
    Interface(KInterface),
}

#[derive(Debug, Clone, PredictiveParse, Spanned)]
pub struct Class {
    pub class: ClassKind,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub within: Option<Within>,
    pub specifiers: Vec<ClassSpecifier>,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Extends {
    pub extends: KExtends,
    pub parent_class: Path,
}

#[derive(Debug, Clone, Parse, PredictiveParse, Spanned)]
pub struct Within {
    pub within: KWithin,
    pub outer_class: Ident,
}

#[derive(Debug, Clone, Parse, Spanned)]
#[parse(error = "specifier_error")]
pub enum ClassSpecifier {
    #[parse(keyword = "abstract")]
    Abstract(Ident),
    #[parse(keyword = "alwaysloaded")]
    AlwaysLoaded(Ident),
    #[parse(keyword = "autoexpandcategories")]
    AutoExpandCategories(Ident, SpecifierArgs),
    #[parse(keyword = "classgroup")]
    ClassGroup(Ident, SpecifierArgs),
    #[parse(keyword = "collapsecategories")]
    CollapseCategories(Ident),
    #[parse(keyword = "config")]
    Config(Ident, SpecifierArgs),
    #[parse(keyword = "dependson")]
    DependsOn(Ident, SpecifierArgs),
    #[parse(keyword = "deprecated")]
    Deprecated(Ident),
    #[parse(keyword = "dontcollapsecategories")]
    DontCollapseCategories(Ident),
    #[parse(keyword = "dontsortcategories")]
    DontSortCategories(Ident, SpecifierArgs),
    #[parse(keyword = "editinlinenew")]
    EditInlineNew(Ident),
    #[parse(keyword = "forcescriptorder")]
    ForceScriptOrder(Ident, LeftParen, BoolLit, RightParen),
    #[parse(keyword = "hidecategories")]
    HideCategories(Ident, SpecifierArgs),
    #[parse(keyword = "hidedropdown")]
    HideDropdown(Ident),
    #[parse(keyword = "implements")]
    Implements(Ident, SpecifierArgs),
    #[parse(keyword = "inherits")]
    Inherits(Ident, SpecifierArgs),
    #[parse(keyword = "iterationoptimized")]
    IterationOptimized(Ident),
    #[parse(keyword = "native")]
    Native(Ident, Option<SpecifierArgs>),
    #[parse(keyword = "nativereplication")]
    NativeReplication(Ident),
    #[parse(keyword = "nevercook")]
    NeverCook(Ident),
    #[parse(keyword = "noexport")]
    NoExport(Ident),
    #[parse(keyword = "notplaceable")]
    NotPlaceable(Ident),
    #[parse(keyword = "perobjectconfig")]
    PerObjectConfig(Ident),
    #[parse(keyword = "placeable")]
    Placeable(Ident),
    #[parse(keyword = "showcategories")]
    ShowCategories(Ident, SpecifierArgs),
    #[parse(keyword = "transient")]
    Transient(Ident),
}

impl Parse for Class {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let class = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error("class name expected")
                .with_label(labels::invalid_identifier(span, &parser.sources))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let extends = parser.parse()?;
        let within = parser.parse()?;
        let (specifiers, semi) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error("missing `;` after class specifier list").with_label(
                        Label::primary(&error.parse.span, "this was expected to be `;`"),
                    ),
                ),
            }
            error.parse
        })?;
        Ok(Self {
            class,
            name,
            extends,
            within,
            specifiers,
            semi,
        })
    }
}

fn class_kind_error(_: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error("`class`, `partial class`, or `interface` expected")
        .with_label(Label::primary(token, ""))
        .with_note("note: the file must start with the kind of type you're declaring")
}

fn specifier_error(parser: &Parser<'_, impl ParseStream>, token: &AnyToken) -> Diagnostic<Token> {
    Diagnostic::error(format!(
        "unknown class specifier `{}`",
        parser.sources.source(token)
    ))
    .with_label(Label::primary(token, "this specifier is not recognized"))
    .with_note("note: notable class specifiers include `placeable` and `abstract`")
}
