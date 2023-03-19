use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use ref_cast::RefCast;

/// ASCII-insensitive identifier.
#[derive(Clone, Copy, RefCast)]
#[repr(transparent)]
pub struct CaseInsensitive<S: ?Sized>(S);

impl<S> CaseInsensitive<S> {
    pub fn new(inner: S) -> Self {
        Self(inner)
    }
}

impl CaseInsensitive<str> {
    pub fn new_ref(s: &str) -> &Self {
        CaseInsensitive::ref_cast(s)
    }
}

impl<S> fmt::Debug for CaseInsensitive<S>
where
    S: ?Sized + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<S> PartialEq for CaseInsensitive<S>
where
    S: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().eq_ignore_ascii_case(other.0.as_ref())
    }
}

impl<S> Eq for CaseInsensitive<S> where S: ?Sized + AsRef<str> {}

impl<S> PartialOrd for CaseInsensitive<S>
where
    S: ?Sized + PartialOrd + AsRef<str>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<S> Ord for CaseInsensitive<S>
where
    S: ?Sized + Ord + AsRef<str>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<S> Hash for CaseInsensitive<S>
where
    S: ?Sized + AsRef<str>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.0
            .as_ref()
            .chars()
            .for_each(|c| c.to_ascii_lowercase().hash(state))
    }
}

impl Borrow<CaseInsensitive<str>> for CaseInsensitive<String> {
    fn borrow(&self) -> &CaseInsensitive<str> {
        CaseInsensitive::ref_cast(&self.0)
    }
}

impl AsRef<str> for CaseInsensitive<String> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for CaseInsensitive<str> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> Deref for CaseInsensitive<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
