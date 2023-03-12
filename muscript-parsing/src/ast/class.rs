use muscript_foundation::errors::{Diagnostic, Label};

use crate::{
    diagnostics::{labels, notes},
    lexis::token::{Ident, LeftParen, RightParen, Semi, Token},
    list::TerminatedListErrorKind,
    Parse, ParseError, ParseStream, Parser, PredictiveParse,
};

use super::{
    BoolLit, KAbstract, KAlwaysLoaded, KAutoExpandCategories, KClassGroup, KCollapseCategories,
    KConfig, KDependsOn, KDeprecated, KDontCollapseCategories, KDontSortCategories, KEditInlineNew,
    KForceScriptOrder, KHideCategories, KHideDropdown, KImplements, KInherits, KIterationOptimized,
    KNative, KNativeReplication, KNeverCook, KNoExport, KNotPlaceable, KPerObjectConfig,
    KPlaceable, KShowCategories, KTransient, Path, SpecifierArgs,
};

keyword! {
    KClass = "class",
    KInterface = "interface",
    KExtends = "extends",
    KWithin = "within",
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
#[parse(error = "class_kind_error")]
pub enum ClassKind {
    Class(KClass),
    Interface(KInterface),
}

#[derive(Debug, Clone, PredictiveParse)]
pub struct Class {
    pub class: ClassKind,
    pub name: Ident,
    pub extends: Option<Extends>,
    pub within: Option<Within>,
    pub specifiers: Vec<ClassSpecifier>,
    pub semi: Semi,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Extends {
    pub extends: KExtends,
    pub parent_class: Path,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct Within {
    pub within: KWithin,
    pub outer_class: Ident,
}

#[derive(Debug, Clone, Parse)]
#[parse(error = "specifier_error")]
pub enum ClassSpecifier {
    Abstract(KAbstract),
    AlwaysLoaded(KAlwaysLoaded),
    AutoExpandCategories(KAutoExpandCategories, SpecifierArgs),
    ClassGroup(KClassGroup, SpecifierArgs),
    CollapseCategories(KCollapseCategories),
    Config(KConfig, SpecifierArgs),
    DependsOn(KDependsOn, SpecifierArgs),
    Deprecated(KDeprecated),
    DontCollapseCategories(KDontCollapseCategories),
    DontSortCategories(KDontSortCategories, SpecifierArgs),
    EditInlineNew(KEditInlineNew),
    ForceScriptOrder(KForceScriptOrder, LeftParen, BoolLit, RightParen),
    HideCategories(KHideCategories, SpecifierArgs),
    HideDropdown(KHideDropdown),
    Implements(KImplements, SpecifierArgs),
    Inherits(KInherits, SpecifierArgs),
    IterationOptimized(KIterationOptimized),
    Native(KNative, Option<SpecifierArgs>),
    NativeReplication(KNativeReplication),
    NeverCook(KNeverCook),
    NoExport(KNoExport),
    NotPlaceable(KNotPlaceable),
    PerObjectConfig(KPerObjectConfig),
    Placeable(KPlaceable),
    ShowCategories(KShowCategories, SpecifierArgs),
    Transient(KTransient),
}

impl Parse for Class {
    fn parse(parser: &mut Parser<'_, impl ParseStream>) -> Result<Self, ParseError> {
        let class = parser.parse()?;
        let name = parser.parse_with_error(|parser, span| {
            Diagnostic::error(parser.file, "class name expected")
                .with_label(labels::invalid_identifier(span, parser.input))
                .with_note(notes::IDENTIFIER_CHARS)
        })?;
        let extends = parser.parse()?;
        let within = parser.parse()?;
        let (specifiers, semi) = parser.parse_terminated_list().map_err(|error| {
            match error.kind {
                TerminatedListErrorKind::Parse => (),
                TerminatedListErrorKind::MissingTerminator => parser.emit_diagnostic(
                    Diagnostic::error(parser.file, "missing `;` after class specifier list")
                        .with_label(Label::primary(
                            error.parse.span,
                            "this was expected to be `;`",
                        )),
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

fn class_kind_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(parser.file, "`class` or `interface` expected")
        .with_label(Label::primary(token.span, ""))
        .with_note("note: the file must start with the kind of type you're declaring")
}

fn specifier_error(parser: &Parser<'_, impl ParseStream>, token: &Token) -> Diagnostic {
    Diagnostic::error(
        parser.file,
        format!(
            "unknown class specifier `{}`",
            token.span.get_input(parser.input)
        ),
    )
    .with_label(Label::primary(
        token.span,
        "this specifier is not recognized",
    ))
    .with_note("note: notable class specifiers include `placeable` and `abstract`")
}
