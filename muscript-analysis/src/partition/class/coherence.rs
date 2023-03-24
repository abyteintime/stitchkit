use indexmap::IndexMap;
use muscript_foundation::{
    errors::DiagnosticSink,
    ident::CaseInsensitive,
    source::{SourceFileId, SourceFileSet, Span},
};
use muscript_syntax::cst::NamedItem;

use super::UntypedClassPartition;

impl UntypedClassPartition {
    /// Coherence check - checks that no identifiers are redeclared across multiple partitions.
    pub fn check_namespace_coherence(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
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
                    partition.source_file_id,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut functions,
                    &partition.functions,
                    partition.source_file_id,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut types,
                    &partition.types,
                    partition.source_file_id,
                );
                Self::check_coherence_in_namespace(
                    diagnostics,
                    sources,
                    &mut states,
                    &partition.states,
                    partition.source_file_id,
                );
            }
        }
    }

    fn check_coherence_in_namespace<I>(
        diagnostics: &mut dyn DiagnosticSink,
        sources: &SourceFileSet,
        joint_namespace: &mut IndexMap<CaseInsensitive<String>, (SourceFileId, Span)>,
        partition_namespace: &IndexMap<CaseInsensitive<String>, I>,
        partition_source_file_id: SourceFileId,
    ) where
        I: NamedItem,
    {
        for (name, var) in partition_namespace {
            if let Some(&(source_file_id_first, span_first)) = joint_namespace.get(name) {
                diagnostics.emit(Self::redeclaration_error(
                    sources,
                    source_file_id_first,
                    span_first,
                    partition_source_file_id,
                    var.name().span,
                ));
            } else {
                joint_namespace.insert(name.clone(), (partition_source_file_id, var.name().span));
            }
        }
    }
}
