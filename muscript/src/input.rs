use std::{collections::HashMap, rc::Rc};

use muscript_analysis::CompilerInput;
use muscript_foundation::{
    errors::{pipe_all_diagnostics_into, DiagnosticSink},
    ident::CaseInsensitive,
    source::{SourceFileId, SourceFileSet},
};
use muscript_syntax::{
    cst,
    lexis::preprocessor::{Definitions, Preprocessor},
    Parser, Structured,
};

pub struct Input<'a> {
    source_file_set: &'a SourceFileSet,
    class_sources: HashMap<CaseInsensitive<String>, Vec<SourceFileId>>,
}

impl<'a> Input<'a> {
    pub fn new(source_file_set: &'a SourceFileSet) -> Self {
        Self {
            source_file_set,
            class_sources: Default::default(),
        }
    }

    pub fn add(&mut self, class_name: &str, source_file: SourceFileId) {
        if let Some(sources) = self
            .class_sources
            .get_mut(CaseInsensitive::new_ref(class_name))
        {
            sources.push(source_file);
        } else {
            self.class_sources.insert(
                CaseInsensitive::new(class_name.to_owned()),
                vec![source_file],
            );
        }
    }
}

impl<'a> CompilerInput for Input<'a> {
    fn class_exists(&self, class_name: &str) -> bool {
        self.class_sources
            .contains_key(CaseInsensitive::new_ref(class_name))
    }

    fn class_sources(
        &self,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink,
    ) -> Option<Vec<(SourceFileId, cst::File)>> {
        self.class_sources
            .get(CaseInsensitive::new_ref(class_name))
            .map(|source_file_ids| {
                source_file_ids
                    .iter()
                    .flat_map(|&id| {
                        let source_file = self.source_file_set.get(id);
                        // TODO: Let these be specified from the outside.
                        let mut definitions = Definitions::default();
                        let mut preprocessor_diagnostics = vec![];
                        let preprocessor = Preprocessor::new(
                            id,
                            Rc::clone(&source_file.source),
                            &mut definitions,
                            &mut preprocessor_diagnostics,
                        );
                        let mut parser_diagnostics = vec![];
                        let mut parser = Parser::new(
                            id,
                            &source_file.source,
                            Structured::new(preprocessor),
                            &mut parser_diagnostics,
                        );
                        let result = parser.parse::<cst::File>();
                        pipe_all_diagnostics_into(diagnostics, preprocessor_diagnostics);
                        pipe_all_diagnostics_into(diagnostics, parser_diagnostics);
                        result.map(|file| (id, file))
                    })
                    .collect()
            })
    }
}
