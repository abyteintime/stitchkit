use muscript_foundation::errors::DiagnosticSink;
use muscript_syntax::cst;

/// External source providing source code for classes.
pub trait CompilerInput {
    /// Returns the parsed sources of a class. May be empty if any errors occur.
    ///
    /// In case `None` is returned, a class with the given name does not exist.
    ///
    /// `diagnostics` should be filled in with any errors that occur during parsing. Files that
    /// irrecoverably fail to parse should not be included in the output.
    fn class_sources(
        &self,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink,
    ) -> Option<Vec<cst::File>>;
}
