use bitflags::bitflags;
use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::{SourceFileId, SourceFileSet, Spanned},
};
use muscript_syntax::cst;

use crate::{
    class::{Var, VarFlags, VarKind},
    diagnostics::notes,
    ir::Ir,
    ClassId, Compiler, FunctionId, TypeId, VarId,
};

use self::builder::FunctionBuilder;

mod builder;
mod expr;
pub mod mangling;
mod stmt;

#[derive(Clone)]
pub struct Function {
    pub source_file_id: SourceFileId,
    pub class: ClassId,
    pub mangled_name: String,
    pub ir: Ir,

    pub return_ty: TypeId,
    pub params: Vec<Param>,

    pub flags: FunctionFlags,
    pub implementation: FunctionImplementation,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct FunctionFlags: u16 {
        const CLIENT               = 0x1;
        const EDITOR_ONLY          = 0x2;
        const EXEC                 = 0x4;
        const EXPENSIVE            = 0x8;
        const FINAL                = 0x10;
        const ITERATOR             = 0x20;
        const LATENT               = 0x40;
        const MULTICAST            = 0x80;
        const NO_OWNER_REPLICATION = 0x400;
        const RELIABLE             = 0x800;
        const SERVER               = 0x1000;
        const SIMULATED            = 0x2000;
        const SINGULAR             = 0x4000;
        const STATIC               = 0x8000;
        // Omitted:
        // - `native`, because that's handled through a different channel (field in Function.)
        // - `coerce`, because it's unclear what it's supposed to mean.
        // - `noexport`, `noexportheader`, `const`, and `virtual`, because we don't support
        //   emitting C++ headers.
    }
}

#[derive(Clone)]
pub struct Param {
    pub var: VarId,
    pub flags: ParamFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ParamFlags: u8 {
        const COERCE   = 0x1;
        const OPTIONAL = 0x2;
        const OUT      = 0x4;
        const SKIP     = 0x8;
    }
}

/// How a function is implemented, and how it should be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionImplementation {
    /// Implemented in UnrealScript. Uses the normal calling convention.
    Script,
    /// Implemented in C++. Uses the normal calling convention.
    Native,
    /// Implemented in C++ as an opcode, using the opcode calling convention.
    Opcode(u16),
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").finish_non_exhaustive()
    }
}

impl<'a> Compiler<'a> {
    /// Analyzes a function from CST.
    pub(crate) fn analyze_function(
        &mut self,
        source_file_id: SourceFileId,
        class_id: ClassId,
        name: &str,
        cst: &cst::ItemFunction,
    ) -> FunctionId {
        let (flags, implementation) = FunctionFlags::from_pre_specifiers(
            self.env,
            self.sources,
            source_file_id,
            &cst.pre_specifiers,
        );

        let return_ty = cst
            .return_ty
            .as_ref()
            .map(|ty| self.type_id(source_file_id, class_id, ty))
            .unwrap_or(TypeId::VOID);

        let mut params = vec![];
        for param in &cst.params.params {
            let (var_flags, param_flags) = flags_from_param_specifiers(&param.specifiers);
            unsupported_param_specifiers(self.env, source_file_id, &param.specifiers);

            let ty = self.type_id(source_file_id, class_id, &param.ty);
            let param_var = self.env.register_var(Var {
                source_file_id,
                name: param.name,
                kind: VarKind::Var {
                    ty,
                    flags: var_flags,
                },
            });
            params.push(Param {
                var: param_var,
                flags: param_flags,
            });
        }

        unsupported_post_specifiers(self.env, source_file_id, &cst.post_specifiers);

        let mut ir_builder = Ir::builder();
        for param in &params {
            ir_builder.add_local(param.var);
        }

        let mut builder = FunctionBuilder {
            source_file_id,
            class_id,
            function_keyword_span: cst.kind.span(),
            flags,
            return_ty,
            params,
            ir: ir_builder,
        };
        match &cst.body {
            cst::Body::Stub(semi) => {
                if !matches!(
                    &implementation,
                    FunctionImplementation::Native | FunctionImplementation::Opcode(_)
                ) {
                    self.env.emit(
                        Diagnostic::error(source_file_id, "function body expected")
                            .with_label(Label::primary(semi.span, ""))
                            .with_note("note: functions can only be stubbed out when they're in interfaces, or when they're `native`"),
                    )
                }
            }
            cst::Body::Impl(block) => {
                self.stmt_block(&mut builder, block);
            }
        }

        self.env
            .register_function(builder.into_function(name.to_owned(), implementation))
    }
}

impl FunctionFlags {
    fn from_pre_specifiers(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        source_file_id: SourceFileId,
        specifiers: &[cst::FunctionSpecifier],
    ) -> (FunctionFlags, FunctionImplementation) {
        let mut flags = FunctionFlags::empty();
        let mut implementation = FunctionImplementation::Script;

        for specifier in specifiers {
            let previous_flags = flags;

            match specifier {
                cst::FunctionSpecifier::Client(_) => flags |= FunctionFlags::CLIENT,
                cst::FunctionSpecifier::EditorOnly(_) => flags |= FunctionFlags::EDITOR_ONLY,
                cst::FunctionSpecifier::Exec(_) => flags |= FunctionFlags::EXEC,
                cst::FunctionSpecifier::Expensive(_) => flags |= FunctionFlags::EXPENSIVE,
                cst::FunctionSpecifier::Final(_) => flags |= FunctionFlags::FINAL,
                cst::FunctionSpecifier::Iterator(_) => flags |= FunctionFlags::ITERATOR,
                cst::FunctionSpecifier::Latent(_) => flags |= FunctionFlags::LATENT,
                cst::FunctionSpecifier::Multicast(_) => flags |= FunctionFlags::MULTICAST,
                cst::FunctionSpecifier::NoOwnerReplication(_) => {
                    flags |= FunctionFlags::NO_OWNER_REPLICATION
                }
                cst::FunctionSpecifier::Reliable(_) => flags |= FunctionFlags::RELIABLE,
                cst::FunctionSpecifier::Server(_) => flags |= FunctionFlags::SERVER,
                cst::FunctionSpecifier::Simulated(_) => flags |= FunctionFlags::SIMULATED,
                cst::FunctionSpecifier::Singular(_) => flags |= FunctionFlags::SINGULAR,
                cst::FunctionSpecifier::Static(_) => flags |= FunctionFlags::STATIC,

                cst::FunctionSpecifier::Native(_, None) => {
                    implementation = FunctionImplementation::Native;
                    // NOTE: For natives, skip the iteration. We don't modify the flags so otherwise
                    // we'll get a warning that `native` was skipped when it in fact, was not.
                    continue;
                }
                cst::FunctionSpecifier::Native(_, Some(opcode_index_cst)) => {
                    let input = sources.source(source_file_id);
                    let opcode_index =
                        opcode_index_cst
                            .number
                            .parse(input, diagnostics, source_file_id);
                    if !(0..=4095).contains(&opcode_index) {
                        diagnostics.emit(
                            Diagnostic::error(source_file_id, "`native` index out of range")
                                .with_label(Label::primary(opcode_index_cst.number.span, ""))
                                .with_note("note: indices of native functions bound to opcodes must lie within the [0, 4095] range"),
                        )
                    }
                    implementation = FunctionImplementation::Opcode(opcode_index as u16);
                    continue;
                }

                cst::FunctionSpecifier::Const(ident) => diagnostics.emit(
                    Diagnostic::error(
                        source_file_id,
                        "`const` specifier must be placed after the function's parameters",
                    )
                    .with_label(Label::primary(ident.span, ""))
                    .with_note(indoc!{"
                        note: even if placed there, `const` is ignored because it's only relevant for exporting
                              C++ headers, which MuScript does not support
                    "}),
                ),

                cst::FunctionSpecifier::Coerce(_) => (),
                cst::FunctionSpecifier::NoExport(_) => (),
                cst::FunctionSpecifier::NoExportHeader(_) => (),
                cst::FunctionSpecifier::Public(_) => (),
                cst::FunctionSpecifier::Private(_) => (),
                cst::FunctionSpecifier::Protected(_) => (),
                cst::FunctionSpecifier::Virtual(_) => (),
            }

            if flags == previous_flags {
                let mut diagnostic = Diagnostic::warning(source_file_id, "specifier is ignored")
                    .with_label(Label::primary(specifier.span(), ""));

                match specifier {
                    cst::FunctionSpecifier::Coerce(_) => {
                        diagnostic = diagnostic.with_note("note: it is unclear what `coerce` should do on returned values, so MuScript ignores it");
                    }
                    cst::FunctionSpecifier::NoExport(_)
                    | cst::FunctionSpecifier::NoExportHeader(_)
                    | cst::FunctionSpecifier::Virtual(_) => {
                        diagnostic = diagnostic.with_note(notes::CPP_UNSUPPORTED);
                    }
                    cst::FunctionSpecifier::Public(_)
                    | cst::FunctionSpecifier::Private(_)
                    | cst::FunctionSpecifier::Protected(_) => {
                        diagnostic = diagnostic.with_note(notes::ACCESS_UNSUPPORTED);
                    }
                    _ => (),
                }

                diagnostics.emit(diagnostic);
            }
        }

        (flags, implementation)
    }
}

fn unsupported_post_specifiers(
    diagnostics: &mut dyn DiagnosticSink,
    source_file_id: SourceFileId,
    specifiers: &[cst::FunctionSpecifier],
) {
    for specifier in specifiers {
        if let cst::FunctionSpecifier::Const(ident) = specifier {
            diagnostics.emit(
                Diagnostic::warning(source_file_id, "`const` specifier is ignored")
                    .with_label(Label::primary(ident.span, ""))
                    .with_note(indoc! {"
                        note: `const` after the function's parameters is only relevant for exporting C++ headers,
                              which MuScript does not support
                    "}),
            )
        } else {
            diagnostics.emit(
                Diagnostic::error(
                    source_file_id,
                    "specifiers other than `const` are not allowed here",
                )
                .with_label(Label::primary(specifier.span(), ""))
                .with_note("help: try placing the specifier before `function`"),
            )
        }
    }
}

fn flags_from_param_specifiers(specifiers: &[cst::ParamSpecifier]) -> (VarFlags, ParamFlags) {
    let mut var = VarFlags::empty();
    let mut param = ParamFlags::empty();

    for specifier in specifiers {
        match specifier {
            cst::ParamSpecifier::Coerce(_) => param |= ParamFlags::COERCE,
            cst::ParamSpecifier::Const(_) => var |= VarFlags::CONST,
            cst::ParamSpecifier::Init(_) => var |= VarFlags::INIT,
            cst::ParamSpecifier::Optional(_) => param |= ParamFlags::OPTIONAL,
            cst::ParamSpecifier::Out(_) => param |= ParamFlags::OUT,
            cst::ParamSpecifier::Skip(_) => param |= ParamFlags::SKIP,
        }
    }

    (var, param)
}

fn unsupported_param_specifiers(
    diagnostics: &mut dyn DiagnosticSink,
    source_file_id: SourceFileId,
    specifiers: &[cst::ParamSpecifier],
) {
    for specifier in specifiers {
        if let cst::ParamSpecifier::Skip(ident) = specifier {
            diagnostics.emit(
                Diagnostic::warning(source_file_id, "`skip` specifier is ignored")
                    .with_label(Label::primary(ident.span, ""))
                    .with_note("note: MuScript currently does not support the `skip` specifier on non-`native` functions"),
            );
        }
    }
}
