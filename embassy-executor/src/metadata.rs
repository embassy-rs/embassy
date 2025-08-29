#[cfg(feature = "metadata-name")]
use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::task::Poll;

#[cfg(feature = "metadata-name")]
use critical_section::Mutex;

use crate::raw;

/// Metadata associated with a task.
pub struct Metadata {
    #[cfg(feature = "metadata-name")]
    name: Mutex<Cell<Option<&'static str>>>,
}

impl Metadata {
    pub(crate) const fn new() -> Self {
        Self {
            #[cfg(feature = "metadata-name")]
            name: Mutex::new(Cell::new(None)),
        }
    }

    pub(crate) fn reset(&self) {
        #[cfg(feature = "metadata-name")]
        critical_section::with(|cs| self.name.borrow(cs).set(None));
    }

    /// Get the metadata for the current task.
    ///
    /// You can use this to read or modify the current task's metadata.
    ///
    /// This function is `async` just to get access to the current async
    /// context. It returns instantly, it does not block/yield.
    pub fn for_current_task() -> impl Future<Output = &'static Self> {
        poll_fn(|cx| Poll::Ready(raw::task_from_waker(cx.waker()).metadata()))
    }

    /// Get this task's name
    ///
    /// NOTE: this takes a critical section.
    #[cfg(feature = "metadata-name")]
    pub fn name(&self) -> Option<&'static str> {
        critical_section::with(|cs| self.name.borrow(cs).get())
    }

    /// Set this task's name
    ///
    /// NOTE: this takes a critical section.
    #[cfg(feature = "metadata-name")]
    pub fn set_name(&self, name: &'static str) {
        critical_section::with(|cs| self.name.borrow(cs).set(Some(name)))
    }
}
