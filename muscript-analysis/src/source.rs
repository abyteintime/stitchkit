use muscript_foundation::{errors::DiagnosticSink, source::SourceFileId};
use muscript_syntax::{cst, lexis::token::Token};

/// Collection of source files for a class.
#[derive(Debug, Clone)]
pub struct ClassSources {
    pub source_files: Vec<ClassSourceFile>,
}

/// A single class source file.
#[derive(Debug, Clone)]
pub struct ClassSourceFile {
    pub id: SourceFileId,
    pub parsed: cst::File,
}

/// External source providing source code for classes.
pub trait CompilerInput {
    /// Returns whether a class with the given name exists.
    fn class_exists(&self, class_name: &str) -> bool;

    /// Returns the source file IDs of a class.
    ///
    /// In case `None` is returned, a class with the given name does not exist.
    fn class_source_ids(&self, class_name: &str) -> Option<Vec<SourceFileId>>;

    /// Returns the parsed sources of a class. May be empty if any errors occur.
    ///
    /// In case `None` is returned, a class with the given name does not exist.
    ///
    /// `diagnostics` should be filled in with any errors that occur during parsing. Files that
    /// irrecoverably fail to parse should not be included in the output.
    fn parsed_class_sources(
        &self,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> Option<ClassSources>;
}
