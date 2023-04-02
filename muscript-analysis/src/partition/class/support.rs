use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::Spanned,
};

use super::UntypedClassPartition;

impl UntypedClassPartition {
    /// Support check - report errors for all items that are unsupported in the current version of
    /// MuScript. Note that this check shouldn't be performed at partitioning time, since that would
    /// go against the philosophy of only analyzing what you use and would thus render a lot more
    /// code uncompilable.
    pub fn check_item_support(&self, diagnostics: &mut dyn DiagnosticSink) {
        let &UntypedClassPartition { source_file_id, .. } = self;

        if let Some(within) = self.within {
            diagnostics.emit(
                Diagnostic::error(
                    source_file_id,
                    "`within` is not yet supported by the compiler",
                )
                .with_label(Label::primary(within.span, "")),
            );
        }

        for state in self.states.values() {
            diagnostics.emit(
                Diagnostic::error(
                    source_file_id,
                    "states are not yet supported by the compiler",
                )
                .with_label(Label::primary(state.span(), "")),
            );
        }
    }
}
