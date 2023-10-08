use std::collections::HashMap;

use muscript_analysis::{ClassSourceFile, ClassSources, CompilerInput};
use muscript_foundation::{errors::DiagnosticSink, ident::CaseInsensitive, source::SourceFileId};
use muscript_lexer::{sources::OwnedSources, token::Token};

use crate::parse::parse_source;

struct Sources {
    source_files: Vec<SourceFileId>,
}

pub struct Input {
    class_sources: HashMap<CaseInsensitive<String>, Sources>,
}

impl Input {
    pub fn new() -> Self {
        Self {
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

impl CompilerInput for Input {
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
        owned_sources: &mut OwnedSources<'_>,
        class_name: &str,
        diagnostics: &mut dyn DiagnosticSink<Token>,
    ) -> Option<ClassSources> {
        self.class_sources
            .get(CaseInsensitive::new_ref(class_name))
            .map(|sources| {
                sources
                    .source_files
                    .iter()
                    .flat_map(|&id| {
                        let result = parse_source(owned_sources, id, diagnostics);
                        result.map(|file| ClassSourceFile { id, parsed: file })
                    })
                    .collect()
            })
            .map(|source_files| ClassSources { source_files })
    }
}
