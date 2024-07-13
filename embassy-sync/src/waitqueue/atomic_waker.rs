use core::task::Waker;

use raw_mutex_traits::BlockingMutex;

use crate::blocking_mutex::raw::CriticalSectionRawMutex;
use crate::blocking_mutex::CriticalSectionMutex;

/// Utility struct to register and wake a waker.
pub struct AtomicWaker {
    waker: CriticalSectionMutex<Option<Waker>>,
}

impl AtomicWaker {
    /// Create a new `AtomicWaker`.
    pub const fn new() -> Self {
        Self {
            waker: CriticalSectionMutex {
                inner: BlockingMutex::const_new(CriticalSectionRawMutex::new(), None),
            },
        }
    }

    /// Register a waker. Overwrites the previous waker, if any.
    pub fn register(&self, w: &Waker) {
        self.waker.lock(|state| {
            *state = match state.take() {
                Some(w2) if (w2.will_wake(w)) => Some(w2),
                Some(w2) => {
                    w2.wake();
                    Some(w.clone())
                }
                _ => Some(w.clone()),
            };
        });
    }

    /// Wake the registered waker, if any.
    pub fn wake(&self) {
        self.waker.lock(|state| {
            if let Some(w) = state.take() {
                w.wake()
            }
        });
    }
}
