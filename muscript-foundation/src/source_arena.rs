use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU32,
};

use crate::{source::SourceFileId, span::Span};

/// ID of an element within a [`SourceArena<T>`].
pub struct SourceId<T> {
    index: NonZeroU32,
    _phantom_data: PhantomData<T>,
}

/// Arena which maps singular elements of source files onto their source file IDs.
///
/// Such elements may include things like syntax tokens, errors, and other things.
#[derive(Debug, Clone)]
pub struct SourceArena<T> {
    source_file_id_mapping: Vec<(SourceId<T>, SourceFileId)>,
    elements: Vec<T>,
}

impl<T> SourceArena<T> {
    pub fn new() -> Self {
        Self {
            source_file_id_mapping: vec![],
            elements: vec![],
        }
    }

    fn current_element_id(&self) -> SourceId<T> {
        SourceId {
            // SAFETY: Always adds 1 to the u32, therefore it can never be zero.
            index: unsafe { NonZeroU32::new_unchecked(self.elements.len() as u32 + 1) },
            _phantom_data: PhantomData,
        }
    }

    pub fn build_source_file(&mut self, source_file_id: SourceFileId) -> SourceArenaBuilder<T> {
        let start = self.current_element_id();
        self.source_file_id_mapping.push((start, source_file_id));
        SourceArenaBuilder {
            source_arena: self,
            start,
        }
    }

    pub fn element(&self, id: SourceId<T>) -> &T {
        &self.elements[(u32::from(id.index) - 1) as usize]
    }

    pub fn source_file_id(&self, id: SourceId<T>) -> SourceFileId {
        match self
            .source_file_id_mapping
            .binary_search_by_key(&id, |&(element_id, _)| element_id)
        {
            Ok(i) => self.source_file_id_mapping[i].1,
            Err(i) => self.source_file_id_mapping[i - 1].1,
        }
    }
}

impl<T> Default for SourceArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SourceArenaBuilder<'a, T> {
    source_arena: &'a mut SourceArena<T>,
    start: SourceId<T>,
}

impl<'a, T> SourceArenaBuilder<'a, T> {
    pub fn push(&mut self, element: T) -> SourceId<T> {
        let id = self.source_arena.current_element_id();
        self.source_arena.elements.push(element);
        id
    }

    pub fn arena(&self) -> &SourceArena<T> {
        self.source_arena
    }

    pub fn finish(self) -> Span<T> {
        let end = self.source_arena.current_element_id();
        Span::Spanning {
            start: self.start,
            end,
        }
    }
}

impl<T> SourceId<T> {
    pub fn successor(self) -> Self {
        Self {
            index: self.index.saturating_add(1),
            _phantom_data: PhantomData,
        }
    }

    pub fn successor_in(self, span: Span<T>) -> Option<Self> {
        match span {
            Span::Empty => None,
            Span::Spanning { end, .. } => (self < end).then_some(self.successor()),
        }
    }
}

impl<T> Debug for SourceId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.index, f)
    }
}

impl<T> Clone for SourceId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SourceId<T> {}

impl<T> PartialEq for SourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for SourceId<T> {}

impl<T> PartialOrd for SourceId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for SourceId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for SourceId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
