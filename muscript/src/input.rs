use std::{collections::HashMap, rc::Rc};

use muscript_analysis::{ClassSourceFile, ClassSources, CompilerInput};
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

struct Sources {
    source_files: Vec<SourceFileId>,
}

pub struct Input<'a> {
    source_file_set: &'a SourceFileSet,
    class_sources: HashMap<CaseInsensitive<String>, Sources>,
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
            sources.source_files.push(source_file);
        } else {
            self.class_sources.insert(
                CaseInsensitive::new(class_name.to_owned()),
                Sources {
                    source_files: vec![source_file],
                },
            );
        }
    }
}

impl<'a> CompilerInput for Input<'a> {
    fn class_exists(&self, class_name: &str) -> bool {
        self.class_sources
            .contains_key(CaseInsensitive::new_ref(class_name))
    }

    fn class_source_ids(&self, class_name: &str) -> Option<Vec<SourceFileId>> {
        self.class_sources
            .get(CaseInsensitive::new_ref(class_name))
            .map(|sources| sources.source_files.clone())
    }

    fn parsed_class_sources(
        &self,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink,
    ) -> Option<ClassSources> {
        self.class_sources
            .get(CaseInsensitive::new_ref(class_name))
            .map(|sources| {
                sources
                    .source_files
                    .iter()
                    .flat_map(|&id| {
                        let source_file = self.source_file_set.get(id);
                        // TODO: Let these be specified from the outside. (#2)
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
                        result.map(|file| ClassSourceFile { id, parsed: file })
                    })
                    .collect()
            })
            .map(|source_files| ClassSources { source_files })
    }
}
