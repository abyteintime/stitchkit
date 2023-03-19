//! Types for representing source code.

use std::{
    fmt,
    ops::{Deref, Range},
    path::{Path, PathBuf},
    rc::Rc,
};

use codespan_reporting::files::Files;
use thiserror::Error;

/// Represents a span of characters within the source code.
///
/// While conceptually this is very similar to [`Range<usize>`], it avoids the huge pitfall of
/// [`Range`] not implementing [`Copy`], and is therefore a lot easier to handle.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const EMPTY: Self = Self { start: 0, end: 0 };

    /// Converts the span to a [`Range`].
    pub fn to_range(self) -> Range<u32> {
        Range::from(self)
    }

    pub fn to_usize_range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// Joins two spans together, forming one big span that includes both `self` and `other`.
    pub fn join(&self, other: &Span) -> Span {
        if *self == Self::EMPTY {
            *other
        } else if *other == Self::EMPTY {
            *self
        } else {
            Span {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            }
        }
    }

    /// Returns the slice of the original input string that this span represents.
    pub fn get_input<'a>(&self, input: &'a str) -> &'a str {
        &input[self.to_usize_range()]
    }
}

impl From<Span> for Range<u32> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

impl From<Range<u32>> for Span {
    fn from(value: Range<u32>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&Range::from(*self), f)
    }
}

/// Implemented by all types that have a source code span attached.
pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl<T> Spanned for Option<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        self.as_ref().map(|x| x.span()).unwrap_or(Span::EMPTY)
    }
}

impl<T> Spanned for Box<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        self.deref().span()
    }
}

impl<T> Spanned for Vec<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        self.first()
            .zip(self.last())
            .map(|(first, last)| first.span().join(&last.span()))
            .unwrap_or(Span::EMPTY)
    }
}

/// Represents a single source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// The source file's pretty name.
    pub filename: String,
    /// The full path to the source file.
    pub full_path: PathBuf,
    /// The source code.
    pub source: Rc<str>,

    line_starts: Vec<usize>,
}

impl SourceFile {
    /// Creates a new [`SourceFile`].
    pub fn new(filename: String, full_path: PathBuf, source: Rc<str>) -> Self {
        Self {
            filename,
            full_path,
            line_starts: codespan_reporting::files::line_starts(&source).collect(),
            source,
        }
    }

    /// Returns the name of the class this source file declares, or [`Err`] if the filename does not
    /// contain a class name or contains invalid UTF-8 characters.
    pub fn class_name(&self) -> Result<&str, ClassNameError<'_>> {
        let stem = self
            .full_path
            .file_stem()
            .ok_or(ClassNameError::InvalidUtf8(&self.full_path))?;
        let stem = stem
            .to_str()
            .ok_or(ClassNameError::InvalidUtf8(&self.full_path))?;
        Ok(stem
            .split_once('.')
            .map(|(name, _part)| name)
            .unwrap_or(stem))
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

#[derive(Debug, Error)]
pub enum ClassNameError<'a> {
    #[error("source file path {0:?} does not contain a file name")]
    NoFilename(&'a Path),
    #[error("source file path {0:?} contains invalid UTF-8")]
    InvalidUtf8(&'a Path),
}

/// A set of source files needed to compile a single package.
#[derive(Debug, Clone, Default)]
pub struct SourceFileSet {
    pub source_files: Vec<SourceFile>,
}

/// Index of a source file inside of a [`SourceFileSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFileId(usize);

impl SourceFileSet {
    /// Creates a new [`SourceFileSet`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a new [`SourceFile`] to the set and returns its ID.
    pub fn add(&mut self, file: SourceFile) -> SourceFileId {
        let id = SourceFileId(self.source_files.len());
        self.source_files.push(file);
        id
    }

    pub fn get(&self, file: SourceFileId) -> &SourceFile {
        &self.source_files[file.0]
    }

    pub fn len(&self) -> usize {
        self.source_files.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates over all source files in this set.
    pub fn iter(&self) -> impl Iterator<Item = (SourceFileId, &'_ SourceFile)> {
        self.source_files
            .iter()
            .enumerate()
            .map(|(index, file)| (SourceFileId(index), file))
    }

    pub fn source(&self, file: SourceFileId, spanned: &impl Spanned) -> &str {
        spanned.span().get_input(&self.get(file).source)
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
