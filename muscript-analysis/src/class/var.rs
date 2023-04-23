use bitflags::bitflags;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::{SourceFileId, SourceFileSet, Spanned},
};
use muscript_syntax::{cst, lexis::token::Ident};

use crate::{diagnostics::notes, TypeId};

#[derive(Debug, Clone)]
pub struct Var {
    pub source_file_id: SourceFileId,
    pub name: Ident,
    pub ty: TypeId,
    pub kind: VarKind,
}

#[derive(Debug, Clone)]
pub enum VarKind {
    Var(VarFlags),

    /// Consts only store the AST of the expression that's to be inlined. When the const is used,
    /// the expression is evaluated at compile-time using the surrounding context.
    ///
    /// `VarKind::Const` is a bit of an oxymoron, but consts reside in the same namespace as other
    /// variables, therefore we want to treat them as equal.
    ///
    /// Note that this is not the same as `Var` with `VarFlags::CONST`, as that's used for actual
    /// variables that cannot be reassigned.
    Const(cst::Expr),
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct VarFlags: u32 {
        const BITWISE             = 0x1;
        const CONFIG              = 0x2;
        const CONST               = 0x4;
        const CROSS_LEVEL_ACTIVE  = 0x8;
        const CROSS_LEVEL_PASSIVE = 0x10;
        const DATA_BINDING        = 0x20;
        const DEPRECATED          = 0x40;
        const DUPLICATE_TRANSIENT = 0x80;
        const EDIT_CONST          = 0x100;
        const EDIT_HIDE           = 0x200;
        const EDIT_FIXED_SIZE     = 0x400;
        const EDIT_INLINE         = 0x800;
        const EDIT_INLINE_USE     = 0x1000;
        const EDITOR_ONLY         = 0x2000;
        const EDIT_TEXT_BOX       = 0x4000;
        const EXPORT              = 0x8000;
        const GLOBAL_CONFIG       = 0x10000;
        const INIT                = 0x20000;
        const INPUT               = 0x40000;
        const INSTANCED           = 0x80000;
        const INTERP              = 0x100000;
        const LOCALIZED           = 0x200000;
        const NO_CLEAR            = 0x400000;
        const NO_EXPORT           = 0x800000;
        const NO_IMPORT           = 0x1000000;
        const NON_TRANSACTIONAL   = 0x2000000;
        const REP_NOTIFY          = 0x4000000;
        const SERIALIZE           = 0x8000000;
        const SERIALIZE_TEXT      = 0x10000000;
        const TRANSIENT           = 0x20000000;
        // NOTE:
        // - private, protected, and public are mutually exclusive access modifiers, hence they are
        //   not flags. Same with protectedwrite and privatewrite.
        //   - Well actually they're currently completely NYI; everything behaves as if it were
        //     public. This is by design, as access levels add another layer of complexity and are
        //     probably not as useful for writing mods.
        // - notforconsole is omitted because we cannot build for consoles.
        // - native is omitted because exporting headers is not supported.
    }
}

impl VarFlags {
    pub fn from_cst(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        specifiers: &[cst::VarSpecifier],
    ) -> Self {
        let mut result = Self::empty();
        for specifier in specifiers {
            let before_modification = result;
            let mut ignored = false;

            match specifier {
                cst::VarSpecifier::BitWise(_) => result |= Self::BITWISE,
                cst::VarSpecifier::Config(_) => result |= Self::CONFIG,
                cst::VarSpecifier::CrossLevelActive(_) => result |= Self::CROSS_LEVEL_ACTIVE,
                cst::VarSpecifier::CrossLevelPassive(_) => result |= Self::CROSS_LEVEL_PASSIVE,
                cst::VarSpecifier::DataBinding(_) => result |= Self::DATA_BINDING,
                cst::VarSpecifier::Deprecated(_) => result |= Self::DEPRECATED,
                cst::VarSpecifier::DuplicateTransient(_) => result |= Self::DUPLICATE_TRANSIENT,
                cst::VarSpecifier::EditConst(_) => result |= Self::EDIT_CONST,
                cst::VarSpecifier::EditHide(_) => result |= Self::EDIT_HIDE,
                cst::VarSpecifier::EditFixedSize(_) => result |= Self::EDIT_FIXED_SIZE,
                cst::VarSpecifier::EditInline(_) => result |= Self::EDIT_INLINE,
                cst::VarSpecifier::EditInlineUse(_) => result |= Self::EDIT_INLINE_USE,
                cst::VarSpecifier::EditorOnly(_) => result |= Self::EDITOR_ONLY,
                cst::VarSpecifier::EditTextBox(_) => result |= Self::EDIT_TEXT_BOX,
                cst::VarSpecifier::Export(_) => result |= Self::EXPORT,
                cst::VarSpecifier::GlobalConfig(_) => result |= Self::GLOBAL_CONFIG,
                cst::VarSpecifier::Init(_) => result |= Self::INIT,
                cst::VarSpecifier::Input(_) => result |= Self::INPUT,
                cst::VarSpecifier::Instanced(_) => result |= Self::INSTANCED,
                cst::VarSpecifier::Interp(_) => result |= Self::INTERP,
                cst::VarSpecifier::Localized(_) => result |= Self::LOCALIZED,
                cst::VarSpecifier::NoClear(_) => result |= Self::NO_CLEAR,
                cst::VarSpecifier::NoExport(_) => result |= Self::NO_EXPORT,
                cst::VarSpecifier::NoImport(_) => result |= Self::NO_IMPORT,
                cst::VarSpecifier::NonTransactional(_) => result |= Self::NON_TRANSACTIONAL,
                cst::VarSpecifier::RepNotify(_) => result |= Self::REP_NOTIFY,
                cst::VarSpecifier::Serialize(_) => result |= Self::SERIALIZE,
                cst::VarSpecifier::SerializeText(_) => result |= Self::SERIALIZE_TEXT,

                cst::VarSpecifier::Type(specifier) => match specifier {
                    cst::TypeSpecifier::Const(_) => result |= Self::CONST,
                    cst::TypeSpecifier::Transient(_) => result |= Self::TRANSIENT,

                    // Native is unimplemented.
                    cst::TypeSpecifier::Native(_) => ignored = true,
                },

                // Unimplemented specifiers.
                cst::VarSpecifier::NotForConsole(_)
                | cst::VarSpecifier::Private(_, _)
                | cst::VarSpecifier::PrivateWrite(_)
                | cst::VarSpecifier::Protected(_, _)
                | cst::VarSpecifier::ProtectedWrite(_, _)
                | cst::VarSpecifier::Public(_, _) => ignored = true,
            }

            if ignored {
                diagnostics.emit({
                    let mut diagnostic =
                        Diagnostic::warning(source_file_id, "specifier is ignored")
                            .with_label(Label::primary(specifier.span(), ""));
                    match specifier {
                        cst::VarSpecifier::NotForConsole(_) => {
                            diagnostic =
                                diagnostic.with_note("note: mods cannot be built for consoles")
                        }
                        cst::VarSpecifier::Private(_, _)
                        | cst::VarSpecifier::PrivateWrite(_)
                        | cst::VarSpecifier::Protected(_, _)
                        | cst::VarSpecifier::ProtectedWrite(_, _)
                        | cst::VarSpecifier::Public(_, _) => {
                            diagnostic = diagnostic.with_note(notes::ACCESS_UNSUPPORTED);
                        }
                        cst::VarSpecifier::Type(cst::TypeSpecifier::Native(_)) => {
                            diagnostic = diagnostic.with_note(notes::CPP_UNSUPPORTED);
                        }

                        _ => (),
                    }
                    diagnostic
                })
            } else if result == before_modification {
                // TODO: Maybe better tracking so that we can show where the specifier occurs first?
                diagnostics.emit(
                    Diagnostic::warning(
                        source_file_id,
                        format!(
                            "repeated `{}` specifier",
                            sources.span(source_file_id, specifier)
                        ),
                    )
                    .with_label(Label::primary(specifier.span(), "")),
                )
            }
        }
        result
    }
}
