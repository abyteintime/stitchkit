use std::{fmt, ops::Deref};

use crate::source_arena::SourceId;

/// Represents a span of elements within the source code.
pub enum Span<T> {
    Empty,
    Spanning {
        start: SourceId<T>,
        end: SourceId<T>,
    },
}

impl<T> Eq for Span<T> {}

impl<T> PartialEq for Span<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start() == other.start() && self.end() == other.end()
    }
}

impl<T> Copy for Span<T> {}

impl<T> Clone for Span<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Span<T> {
    pub fn single(start_and_end: SourceId<T>) -> Self {
        Self::Spanning {
            start: start_and_end,
            end: start_and_end,
        }
    }

    pub fn start(&self) -> Option<SourceId<T>> {
        if let Span::Spanning { start, .. } = self {
            Some(*start)
        } else {
            None
        }
    }

    pub fn end(&self) -> Option<SourceId<T>> {
        if let Span::Spanning { end, .. } = self {
            Some(*end)
        } else {
            None
        }
    }

    /// Joins two spans together, forming one big span that includes both `self` and `other`.
    pub fn join(&self, other: &Self) -> Self {
        match (*self, *other) {
            (Span::Empty, Span::Empty) => todo!(),
            (Span::Empty, Span::Spanning { start, end })
            | (Span::Spanning { start, end }, Span::Empty) => Span::Spanning { start, end },
            (
                Span::Spanning {
                    start: a_start,
                    end: a_end,
                },
                Span::Spanning {
                    start: b_start,
                    end: b_end,
                },
            ) => Span::Spanning {
                start: a_start.min(b_start),
                end: a_end.max(b_end),
            },
        }
    }
}

impl<T> fmt::Debug for Span<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Spanning { start, end } => fmt::Debug::fmt(&(start..end), f),
        }
    }
}

/// Implemented by all types that have a source code span attached.
pub trait Spanned<T> {
    fn span(&self) -> Span<T>;
}

impl<T> Spanned<T> for Span<T> {
    fn span(&self) -> Span<T> {
        *self
    }
}

impl<T, S> Spanned<T> for Option<S>
where
    S: Spanned<T>,
{
    fn span(&self) -> Span<T> {
        self.as_ref().map(|x| x.span()).unwrap_or(Span::Empty)
    }
}

impl<T, S> Spanned<T> for Box<S>
where
    S: Spanned<T>,
{
    fn span(&self) -> Span<T> {
        self.deref().span()
    }
}

impl<T, S> Spanned<T> for Vec<S>
where
    S: Spanned<T>,
{
    fn span(&self) -> Span<T> {
        self.first()
            .zip(self.last())
            .map(|(first, last)| first.span().join(&last.span()))
            .unwrap_or(Span::Empty)
    }
}
