use std::rc::Rc;

use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, DiagnosticSink},
    source::{SourceFileId, SourceFileSet},
};
use muscript_syntax::{
    lexis::preprocessor::{Definitions, Preprocessor},
    Parse, Parser, Structured,
};
use tracing::info_span;

pub fn parse_source<T>(
    source_file_set: &SourceFileSet,
    id: SourceFileId,
    diagnostics: &mut dyn DiagnosticSink,
    definitions: &mut Definitions,
) -> Result<T, muscript_syntax::ParseError>
where
    T: Parse,
{
    let source_file = source_file_set.get(id);
    let _span = info_span!("parse_source", source_file.filename).entered();
    // TODO: Let these be specified from the outside. (#2)
    let mut preprocessor_diagnostics = vec![];
    let preprocessor = Preprocessor::new(
        id,
        Rc::clone(&source_file.source),
        definitions,
        &mut preprocessor_diagnostics,
    );
    let mut parser_diagnostics = vec![];
    let mut parser = Parser::new(
        id,
        &source_file.source,
        Structured::new(preprocessor),
        &mut parser_diagnostics,
    );
    let result = parser.parse::<T>();
    pipe_all_diagnostics_into(diagnostics, preprocessor_diagnostics);
    pipe_all_diagnostics_into(diagnostics, parser_diagnostics);
    result
}
