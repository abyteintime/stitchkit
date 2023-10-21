use indexmap::IndexMap;
use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    source::SourceFileSet,
};
use muscript_lexer::{
    sources::LexedSources,
    token::{Token, TokenSpan},
};
use muscript_syntax::cst::NamedItem;

use crate::ClassSources;

use super::UntypedClassPartition;

impl UntypedClassPartition {
    /// Coherence check - checks that no identifiers are redeclared across multiple partitions.
    pub fn check_namespace_coherence(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &LexedSources<'_>,
        partitions: &[UntypedClassPartition],
    ) {
        // No need to perform the check across a single partition, because that's guaranteed to
        // be coherent by nature (the checks are performed at partitioning time.)
        if partitions.len() >= 2 {
            let mut vars = IndexMap::new();
            let mut functions = IndexMap::new();
            let mut types = IndexMap::new();
            let mut states = IndexMap::new();
            for partition in partitions {
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut vars,
                    &partition.vars,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut functions,
                    &partition.functions,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut types,
                    &partition.types,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut states,
                    &partition.states,
                );
            }
        }
    }

    fn check_coherence_in_namespace<I>(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &LexedSources<'_>,
        joint_namespace: &mut IndexMap<CaseInsensitive<String>, TokenSpan>,
        partition_namespace: &IndexMap<CaseInsensitive<String>, I>,
    ) where
        I: NamedItem,
    {
        for (name, var) in partition_namespace {
            if let Some(&span_first) = joint_namespace.get(name) {
                diagnostics.emit(Self::redeclaration_error(
                    sources,
                    span_first,
                    var.name().span,
                ));
            } else {
                joint_namespace.insert(name.clone(), var.name().span);
            }
        }
    }

    pub fn check_package_coherence(
        diagnostics: &mut dyn DiagnosticSink<Token>,
        sources: &SourceFileSet,
        class_sources: &ClassSources,
    ) {
        let Some(first_source_file) = class_sources.source_files.get(0) else {
            // This can happen if all the class's source files failed to parse.
            return;
        };
        let first_source_file = first_source_file.id;
        let first_source_package = &sources.get(first_source_file).package;

        let mut conflicting = vec![];
        for i in 1..class_sources.source_files.len() {
            let other_source_file = class_sources.source_files[i].id;
            let other_source_package = &sources.get(other_source_file).package;
            if first_source_package != other_source_package {
                conflicting.push(i);
            }
        }

        if !conflicting.is_empty() {
            let mut diagnostic =
                Diagnostic::error("redefinition of class across different packages");
            conflicting.insert(0, 0);
            for i in conflicting {
                let conflicting_cst = &class_sources.source_files[i].parsed;
                diagnostic
                    .labels
                    .push(Label::primary(&conflicting_cst.class.name, ""));
            }
            dbg!(&diagnostic);
            diagnostics.emit(
                diagnostic
                    .with_note("different classes with the same name are defined in multiple packages")
                    .with_note("note: external classes cannot be extended with new behavior because that would require modifying existing game files, which mods cannot do")
                    .with_note("help: try renaming your class to something else"),
            );
        }
    }
}
