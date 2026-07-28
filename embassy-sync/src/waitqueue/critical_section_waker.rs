use core::task::Waker;

use super::GenericAtomicWaker;
use crate::blocking_mutex::raw::CriticalSectionRawMutex;

/// Utility struct to register and wake a waker.
pub struct CriticalSectionWaker {
    waker: GenericAtomicWaker<CriticalSectionRawMutex>,
}

impl CriticalSectionWaker {
    /// Create a new `CriticalSectionWaker`.
    pub const fn new() -> Self {
        Self {
            waker: GenericAtomicWaker::new(CriticalSectionRawMutex::new()),
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        self.waker.register(w);
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        self.waker.wake();
    }
}
