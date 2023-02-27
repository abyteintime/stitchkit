use codespan_reporting::files::Files;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub package: String,
    pub filename: String,
    pub source: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(package: String, filename: String, source: String) -> Self {
        Self {
            package,
            filename,
            line_starts: codespan_reporting::files::line_starts(&source).collect(),
            source,
        }
    }

    fn line_start(&self, line_index: usize) -> Result<usize, codespan_reporting::files::Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self
                .line_starts
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source.len()),
            Ordering::Greater => Err(codespan_reporting::files::Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SourceFileSet {
    pub source_files: Vec<SourceFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceFileId(usize);

impl SourceFileSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, file: SourceFile) -> SourceFileId {
        let id = SourceFileId(self.source_files.len());
        self.source_files.push(file);
        id
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceFileId, &'_ SourceFile)> {
        self.source_files
            .iter()
            .enumerate()
            .map(|(index, file)| (SourceFileId(index), file))
    }
}

impl<'f> Files<'f> for SourceFileSet {
    type FileId = SourceFileId;
    type Name = &'f str;
    type Source = &'f str;

    fn name(&'f self, id: Self::FileId) -> Result<Self::Name, codespan_reporting::files::Error> {
        Ok(&self.source_files[id.0].filename)
    }

    fn source(
        &'f self,
        id: Self::FileId,
    ) -> Result<Self::Source, codespan_reporting::files::Error> {
        Ok(&self.source_files[id.0].source)
    }

    fn line_index(
        &'f self,
        id: Self::FileId,
        byte_index: usize,
    ) -> Result<usize, codespan_reporting::files::Error> {
        Ok(self.source_files[id.0]
            .line_starts
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1))
    }

    fn line_range(
        &'f self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, codespan_reporting::files::Error> {
        let file = &self.source_files[id.0];
        let line_start = file.line_start(line_index)?;
        let next_line_start = file.line_start(line_index + 1)?;
        Ok(line_start..next_line_start)
    }
}
