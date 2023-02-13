//! Thread-local context variables.

use std::cell::Cell;

#[doc(hidden)]
pub struct Context<T> {
    pointer: Cell<*const T>,
}

impl<T> Context<T> {
    pub const fn new() -> Self {
        Self {
            pointer: Cell::new(std::ptr::null()),
        }
    }

    pub fn get<'a>(&self) -> Option<&'a T> {
        if self.pointer.get().is_null() {
            None
        } else {
            Some(unsafe { &*self.pointer.get() })
        }
    }

    pub fn with<R>(&self, reference: &T, then: impl FnOnce() -> R) -> R {
        let previous = self.pointer.get();
        self.pointer.set(reference as *const T);
        let result = then();
        self.pointer.set(previous);
        result
    }
}

#[macro_export]
macro_rules! context {
    (
        $vis:vis let $name:tt : $T:ty ;
    ) => {
        $vis mod $name {
            use super::*;

            thread_local! {
                static CONTEXT: $crate::context::Context<$T> =
                    $crate::context::Context::new();
            }

            pub fn get<'a>() -> Option<&'a $T> {
                CONTEXT.with(|context| context.get())
            }

            pub fn with<R>(reference: &$T, then: impl FnOnce() -> R) -> R {
                CONTEXT.with(|context| context.with(reference, then))
            }
        }
    };
}
